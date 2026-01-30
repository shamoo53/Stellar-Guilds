import { Injectable, NotFoundException, ForbiddenException, BadRequestException, ConflictException, InternalServerErrorException } from '@nestjs/common';
import { validateAndNormalizeSettings } from './guild.settings';
import { PrismaService } from '../prisma/prisma.service';
import { MailerService } from '../mailer/mailer.service';
import { CreateGuildDto } from './dto/create-guild.dto';
import { UpdateGuildDto } from './dto/update-guild.dto';
import { InviteMemberDto } from './dto/invite-member.dto';
import { randomUUID } from 'crypto';

@Injectable()
export class GuildService {
  constructor(private prisma: PrismaService, private mailer: MailerService) {}

  private slugify(name: string) {
    return name
      .toLowerCase()
      .replace(/\s+/g, '-')
      .replace(/[^a-z0-9-]/g, '')
      .substring(0, 100);
  }

  async createGuild(dto: CreateGuildDto, ownerId: string) {
    const slug = dto.slug ? dto.slug : this.slugify(dto.name);

    // Pre-check slug uniqueness to provide friendlier error
    const existing = await this.prisma.guild.findUnique({ where: { slug } });
    if (existing) throw new ConflictException('Slug already in use');

    const normalizedSettings = validateAndNormalizeSettings((dto as any).settings);

    let guild;
    try {
      guild = await this.prisma.guild.create({
        data: {
          name: dto.name,
          slug,
          description: dto.description,
          ownerId,
          settings: normalizedSettings,
        },
      });
    } catch (err: any) {
      // Handle Prisma unique constraint race or other DB errors
      if (err.code === 'P2002') {
        throw new ConflictException('Guild with that slug or unique field already exists');
      }
      throw new InternalServerErrorException('Failed to create guild');
    }

    // create owner membership
    await this.prisma.guildMembership.create({
      data: {
        userId: ownerId,
        guildId: guild.id,
        role: 'OWNER',
        status: 'APPROVED',
        joinedAt: new Date(),
      },
    });

    return guild;
  }

  async getGuild(id: string) {
    const guild = await this.prisma.guild.findUnique({
      where: { id },
      include: { memberships: { include: { user: true } } },
    });
    if (!guild) throw new NotFoundException('Guild not found');
    return guild;
  }

  async getBySlug(slug: string) {
    const guild = await this.prisma.guild.findUnique({
      where: { slug },
      include: { memberships: { include: { user: true } } },
    });
    if (!guild) throw new NotFoundException('Guild not found');
    return guild;
  }

  private async ensureManagePermission(guildId: string, userId: string) {
    const guild = await this.prisma.guild.findUnique({ where: { id: guildId } });
    if (!guild) throw new NotFoundException('Guild not found');
    if (guild.ownerId === userId) return; // owner always allowed

    const membership = await this.prisma.guildMembership.findUnique({
      where: { userId_guildId: { userId, guildId } },
    });
    if (!membership) throw new ForbiddenException('Not a member');

    const weight = this.getRoleWeight(membership.role);
    if (weight <= this.getRoleWeight('MEMBER')) throw new ForbiddenException('Insufficient guild permissions');
  }

  private getRoleWeight(role: string) {
    switch (role) {
      case 'OWNER':
        return 4;
      case 'ADMIN':
        return 3;
      case 'MODERATOR':
        return 2;
      case 'MEMBER':
      default:
        return 1;
    }
  }

  async updateGuild(guildId: string, dto: UpdateGuildDto, userId: string) {
    await this.ensureManagePermission(guildId, userId);
    // If settings present, validate and merge with existing
    const data: any = { ...dto };
    if ((dto as any).settings) {
      const existing = await this.prisma.guild.findUnique({ where: { id: guildId } });
      const validated = validateAndNormalizeSettings((dto as any).settings);
      data.settings = { ...existing.settings, ...validated };
    }

    return this.prisma.guild.update({ where: { id: guildId }, data });
  }

  async deleteGuild(guildId: string, userId: string) {
    const guild = await this.prisma.guild.findUnique({ where: { id: guildId } });
    if (!guild) throw new NotFoundException('Guild not found');
    if (guild.ownerId !== userId) throw new ForbiddenException('Only owner can delete the guild');
    return this.prisma.guild.delete({ where: { id: guildId } });
  }

  async searchGuilds(q: string | undefined, page = 0, size = 20) {
    const textFilter = q
      ? {
          OR: [
            { name: { contains: q, mode: 'insensitive' } },
            { description: { contains: q, mode: 'insensitive' } },
          ],
        }
      : {};

    // Enforce discoverable guilds only for public search
    const discoverFilter = { settings: { path: ['discoverable'], equals: true } };

    const where = Object.keys(textFilter).length ? { AND: [textFilter, discoverFilter] } : discoverFilter;

    const [items, total] = await Promise.all([
      this.prisma.guild.findMany({ where, skip: page * size, take: size }),
      this.prisma.guild.count({ where }),
    ]);

    return { items, total, page, size };
  }

  async inviteMember(guildId: string, dto: InviteMemberDto, invitedBy: string) {
    await this.ensureManagePermission(guildId, invitedBy);
    const existing = await this.prisma.guildMembership.findUnique({ where: { userId_guildId: { userId: dto.userId, guildId } } });
    if (existing) throw new BadRequestException('User already invited or member');

    const token = randomUUID();

    const ttlDays = process.env.INVITE_TTL_DAYS ? Number(process.env.INVITE_TTL_DAYS) : 7;
    const expiresAt = new Date(Date.now() + ttlDays * 24 * 60 * 60 * 1000);

    const membership = await this.prisma.guildMembership.create({ data: {
      userId: dto.userId,
      guildId,
      role: (dto.role as any) || 'MEMBER',
      status: 'PENDING',
      invitationToken: token,
      invitedById: invitedBy,
      invitationExpiresAt: expiresAt,
    }});

    // Try to send invite email to user if email is available
    try {
      const user = await this.prisma.user.findUnique({ where: { id: dto.userId } });
      if (user?.email) {
        const guild = await this.prisma.guild.findUnique({ where: { id: guildId } });
        await this.mailer.sendInviteEmail(user.email, guild?.name || 'a guild', token, undefined);
      }
    } catch (err) {
      // don't fail invite creation on email errors
    }

    return { membership, token };
  }

  async revokeInviteByToken(guildId: string, token: string, revokedBy: string) {
    const membership = await this.prisma.guildMembership.findFirst({ where: { guildId, invitationToken: token } });
    if (!membership) throw new NotFoundException('Invite not found');

    await this.ensureManagePermission(guildId, revokedBy);

    const updated = await this.prisma.guildMembership.update({ where: { id: membership.id }, data: { status: 'REVOKED', invitationToken: null } });

    // notify user
    try {
      const user = await this.prisma.user.findUnique({ where: { id: membership.userId } });
      const guild = await this.prisma.guild.findUnique({ where: { id: guildId } });
      if (user?.email) await this.mailer.sendRevokeEmail(user.email, guild?.name || 'a guild', undefined);
    } catch (_) {}

    return updated;
  }

  async revokeInviteForUser(guildId: string, userId: string, revokedBy: string) {
    const membership = await this.prisma.guildMembership.findUnique({ where: { userId_guildId: { userId, guildId } } });
    if (!membership) throw new NotFoundException('Invite not found');

    // Only allow inviter or guild manager to revoke; or allow the user to cancel their own invite
    if (revokedBy !== userId) await this.ensureManagePermission(guildId, revokedBy);

    const updated = await this.prisma.guildMembership.update({ where: { id: membership.id }, data: { status: 'REVOKED', invitationToken: null } });

    try {
      const user = await this.prisma.user.findUnique({ where: { id: userId } });
      const guild = await this.prisma.guild.findUnique({ where: { id: guildId } });
      if (user?.email) await this.mailer.sendRevokeEmail(user.email, guild?.name || 'a guild', undefined);
    } catch (_) {}

    return updated;
  }

  async approveInviteByToken(guildId: string, token: string, approverId: string) {
    const membership = await this.prisma.guildMembership.findFirst({ where: { guildId, invitationToken: token } });
    if (!membership) throw new NotFoundException('Invite not found');

    // If approver is the invitee, allow; otherwise check permission
    if (membership.userId !== approverId) await this.ensureManagePermission(guildId, approverId);

    const updated = await this.prisma.guildMembership.update({
      where: { id: membership.id },
      data: { status: 'APPROVED', joinedAt: new Date(), invitationToken: null },
    });

    await this.prisma.guild.update({ where: { id: guildId }, data: { memberCount: { increment: 1 } as any } as any });

    return updated;
  }

  async approveInviteForUser(guildId: string, userId: string) {
    const membership = await this.prisma.guildMembership.findUnique({ where: { userId_guildId: { userId, guildId } } });
    if (!membership) throw new NotFoundException('Invite not found');
    if (membership.status !== 'PENDING') throw new BadRequestException('No pending invite to approve');

    const updated = await this.prisma.guildMembership.update({ where: { id: membership.id }, data: { status: 'APPROVED', joinedAt: new Date(), invitationToken: null } });
    await this.prisma.guild.update({ where: { id: guildId }, data: { memberCount: { increment: 1 } as any } as any });
    return updated;
  }

  async joinGuild(guildId: string, userId: string) {
    const existing = await this.prisma.guildMembership.findUnique({ where: { userId_guildId: { userId, guildId } } });
    if (existing && existing.status === 'APPROVED') return existing;
    if (existing && existing.status === 'PENDING') {
      const updated = await this.prisma.guildMembership.update({ where: { id: existing.id }, data: { status: 'APPROVED', joinedAt: new Date() } });
      await this.prisma.guild.update({ where: { id: guildId }, data: { memberCount: { increment: 1 } as any } as any });
      return updated;
    }

    const created = await this.prisma.guildMembership.create({ data: { userId, guildId, role: 'MEMBER', status: 'APPROVED', joinedAt: new Date() } });
    await this.prisma.guild.update({ where: { id: guildId }, data: { memberCount: { increment: 1 } as any } as any });
    return created;
  }

  async leaveGuild(guildId: string, userId: string) {
    const membership = await this.prisma.guildMembership.findUnique({ where: { userId_guildId: { userId, guildId } } });
    if (!membership) throw new NotFoundException('Not a member');
    if (membership.role === 'OWNER') throw new BadRequestException('Owner cannot leave the guild');
    await this.prisma.guildMembership.delete({ where: { id: membership.id } });
    await this.prisma.guild.update({ where: { id: guildId }, data: { memberCount: { decrement: 1 } as any } as any });
    return { success: true };
  }

  async assignRole(guildId: string, targetUserId: string, role: string, byUserId: string) {
    // Ensure actor has management permission
    await this.ensureManagePermission(guildId, byUserId);

    const actorMembership = await this.prisma.guildMembership.findUnique({ where: { userId_guildId: { userId: byUserId, guildId } } });
    if (!actorMembership) throw new ForbiddenException('You are not a member');

    const targetMembership = await this.prisma.guildMembership.findUnique({ where: { userId_guildId: { userId: targetUserId, guildId } } });
    if (!targetMembership) throw new NotFoundException('Member not found');

    // Prevent changing owner's role or assigning OWNER through this endpoint
    if (targetMembership.role === 'OWNER' || role === 'OWNER') throw new BadRequestException('Cannot assign owner role via this endpoint');

    const actorWeight = this.getRoleWeight(actorMembership.role);
    const targetWeight = this.getRoleWeight(targetMembership.role);
    const requestedWeight = this.getRoleWeight(role);

    // Actor must have strictly higher weight than target and must be higher than requested role
    if (actorWeight <= targetWeight) throw new ForbiddenException('Cannot change role of equal or higher member');
    if (actorWeight <= requestedWeight) throw new ForbiddenException('Cannot assign a role equal or higher than your own');

    return this.prisma.guildMembership.update({ where: { id: targetMembership.id }, data: { role: role as any } });
  }
}
