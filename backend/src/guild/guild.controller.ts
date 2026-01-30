import {
  Controller,
  Post,
  Body,
  UseGuards,
  Request,
  Get,
  Param,
  Query,
  Patch,
  Delete,
} from '@nestjs/common';
import { JwtAuthGuard } from '../auth/guards/jwt-auth.guard';
import { GuildRoleGuard } from './guards/guild-role.guard';
import { GuildRoles } from './decorators/guild-roles.decorator';
import { GuildService } from './guild.service';
import { CreateGuildDto } from './dto/create-guild.dto';
import { UpdateGuildDto } from './dto/update-guild.dto';
import { InviteMemberDto } from './dto/invite-member.dto';
import { ApproveInviteDto } from './dto/approve-invite.dto';
import { SearchGuildDto } from './dto/search-guild.dto';

@Controller('guilds')
export class GuildController {
  constructor(private guildService: GuildService) {}

  @UseGuards(JwtAuthGuard)
  @Post()
  async create(@Body() dto: CreateGuildDto, @Request() req: any) {
    return this.guildService.createGuild(dto, req.user.userId);
  }

  @Get(':id')
  async get(@Param('id') id: string) {
    return this.guildService.getGuild(id);
  }

  @Get('by-slug/:slug')
  async getBySlug(@Param('slug') slug: string) {
    return this.guildService.getBySlug(slug);
  }

  @Get()
  async search(@Query() query: SearchGuildDto) {
    return this.guildService.searchGuilds(query.q, query.page, query.size);
  }

  @UseGuards(JwtAuthGuard)
  @Patch(':id')
  @UseGuards(GuildRoleGuard)
  @GuildRoles('ADMIN', 'OWNER')
  async update(@Param('id') id: string, @Body() dto: UpdateGuildDto, @Request() req: any) {
    return this.guildService.updateGuild(id, dto, req.user.userId);
  }

  @UseGuards(JwtAuthGuard)
  @Delete(':id')
  @UseGuards(GuildRoleGuard)
  @GuildRoles('OWNER')
  async remove(@Param('id') id: string, @Request() req: any) {
    return this.guildService.deleteGuild(id, req.user.userId);
  }

  @UseGuards(JwtAuthGuard)
  @Post(':id/invite')
  @UseGuards(GuildRoleGuard)
  @GuildRoles('MODERATOR', 'ADMIN', 'OWNER')
  async invite(@Param('id') id: string, @Body() dto: InviteMemberDto, @Request() req: any) {
    return this.guildService.inviteMember(id, dto, req.user.userId);
  }

  @UseGuards(JwtAuthGuard)
  @Post(':id/revoke')
  @UseGuards(GuildRoleGuard)
  @GuildRoles('MODERATOR', 'ADMIN', 'OWNER')
  async revoke(@Param('id') id: string, @Body() body: any, @Request() req: any) {
    // body may contain token or userId
    if (body.token) return this.guildService.revokeInviteByToken(id, body.token, req.user.userId);
    if (body.userId) return this.guildService.revokeInviteForUser(id, body.userId, req.user.userId);
    return { error: 'token or userId required' };
  }

  @UseGuards(JwtAuthGuard)
  @Post(':id/resend-invite')
  @UseGuards(GuildRoleGuard)
  @GuildRoles('MODERATOR', 'ADMIN', 'OWNER')
  async resendInvite(@Param('id') id: string, @Body() body: any, @Request() req: any) {
    // body must contain userId
    const userId = body.userId;
    if (!userId) return { error: 'userId required' };

    const membership = await this.guildService['prisma'].guildMembership.findUnique({ where: { userId_guildId: { userId, guildId: id } } });
    if (!membership) return { error: 'Invite not found' };
    if (membership.status !== 'PENDING') return { error: 'Not a pending invite' };

    // resend the invite token by email
    const token = membership.invitationToken;
    const user = await this.guildService['prisma'].user.findUnique({ where: { id: userId } });
    const guild = await this.guildService['prisma'].guild.findUnique({ where: { id } });
    if (user?.email && token) {
      await (this.guildService as any).mailer.sendInviteEmail(user.email, guild?.name || 'a guild', token, undefined);
      return { success: true };
    }
    return { error: 'No email or token to resend' };
  }

  @UseGuards(JwtAuthGuard)
  @Post(':id/approve')
  async approve(@Param('id') id: string, @Body() dto: ApproveInviteDto, @Request() req: any) {
    if (dto.token) return this.guildService.approveInviteByToken(id, dto.token, req.user.userId);
    // If no token provided, try to approve the pending invite for the requester
    return this.guildService.approveInviteForUser(id, req.user.userId);
  }

  @UseGuards(JwtAuthGuard)
  @Post(':id/join')
  async join(@Param('id') id: string, @Request() req: any) {
    return this.guildService.joinGuild(id, req.user.userId);
  }

  @UseGuards(JwtAuthGuard)
  @Post(':id/leave')
  async leave(@Param('id') id: string, @Request() req: any) {
    return this.guildService.leaveGuild(id, req.user.userId);
  }

  @UseGuards(JwtAuthGuard)
  @Post(':id/assign-role/:userId')
  @UseGuards(GuildRoleGuard)
  @GuildRoles('ADMIN', 'OWNER')
  async assignRole(@Param('id') id: string, @Param('userId') userId: string, @Body() body: any, @Request() req: any) {
    return this.guildService.assignRole(id, userId, body.role, req.user.userId);
  }
}
