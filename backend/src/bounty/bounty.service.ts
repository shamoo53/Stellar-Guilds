import {
  Injectable,
  NotFoundException,
  ForbiddenException,
  BadRequestException,
} from '@nestjs/common';
import { PrismaService } from '../prisma/prisma.service';
import { MailerService } from '../mailer/mailer.service';
import { CreateBountyDto } from './dto/create-bounty.dto';
import { UpdateBountyDto } from './dto/update-bounty.dto';
import { ApplyBountyDto } from './dto/apply-bounty.dto';
import { CreateMilestoneDto } from './dto/create-milestone.dto';

@Injectable()
export class BountyService {
  constructor(
    private prisma: PrismaService,
    private mailer: MailerService,
  ) {}

  async create(dto: CreateBountyDto, creatorId: string) {
    const data: any = {
      title: dto.title,
      description: dto.description,
      rewardAmount: dto.rewardAmount || 0,
      rewardToken: dto.rewardToken || 'STELLAR',
      creatorId,
      guildId: dto.guildId || null,
    };
    if (dto.deadline) data.deadline = new Date(dto.deadline);
    return this.prisma.bounty.create({ data });
  }

  async get(id: string) {
    const bounty = await this.prisma.bounty.findUnique({
      where: { id },
      include: { creator: true, assignee: true },
    });
    if (!bounty) throw new NotFoundException('Bounty not found');
    return bounty;
  }

  async search(q?: string, page = 0, size = 20, guildId?: string) {
    const text = q
      ? {
          OR: [
            { title: { contains: q, mode: 'insensitive' } },
            { description: { contains: q, mode: 'insensitive' } },
          ],
        }
      : {};

    const where: any = {};
    if (Object.keys(text).length) where.AND = [text];
    if (guildId) where.guildId = guildId;

    const [items, total] = await Promise.all([
      this.prisma.bounty.findMany({ where, skip: page * size, take: size }),
      this.prisma.bounty.count({ where }),
    ]);
    return { items, total, page, size };
  }

  async update(id: string, dto: UpdateBountyDto, userId: string) {
    const bounty = await this.prisma.bounty.findUnique({ where: { id } });
    if (!bounty) throw new NotFoundException('Bounty not found');
    if (bounty.creatorId !== userId)
      throw new ForbiddenException('Only creator can update bounty');
    const data: any = { ...dto };
    if (dto.deadline) data.deadline = new Date(dto.deadline);
    return this.prisma.bounty.update({ where: { id }, data });
  }

  async cancel(id: string, userId: string) {
    const bounty = await this.prisma.bounty.findUnique({ where: { id } });
    if (!bounty) throw new NotFoundException('Bounty not found');
    if (bounty.creatorId !== userId)
      throw new ForbiddenException('Only creator can cancel bounty');
    return this.prisma.bounty.update({
      where: { id },
      data: { status: 'CANCELLED' },
    });
  }

  async apply(bountyId: string, dto: ApplyBountyDto, applicantId: string) {
    const bounty = await this.prisma.bounty.findUnique({
      where: { id: bountyId },
    });
    if (!bounty) throw new NotFoundException('Bounty not found');
    if (bounty.status !== 'OPEN')
      throw new BadRequestException('Bounty not open for applications');

    const existing = await this.prisma.bountyApplication.findFirst({
      where: { bountyId, applicantId },
    });
    if (existing) throw new BadRequestException('Already applied');

    const app = await this.prisma.bountyApplication.create({
      data: {
        bountyId,
        applicantId,
        message: dto.message || null,
        attachments: dto.attachments || null,
      },
    });

    // notify bounty creator
    try {
      const creator = await this.prisma.user.findUnique({
        where: { id: bounty.creatorId },
      });
      if (creator?.email)
        await this.mailer.sendInviteEmail(
          creator.email,
          `New application for bounty ${bounty.title}`,
          app.id,
          undefined,
        );
    } catch (_) {}

    return app;
  }

  async listApplications(bountyId: string, userId: string) {
    const bounty = await this.prisma.bounty.findUnique({
      where: { id: bountyId },
    });
    if (!bounty) throw new NotFoundException('Bounty not found');
    // only creator or guild manager may list; for simplicity allow creator
    if (bounty.creatorId !== userId)
      throw new ForbiddenException('Not allowed');
    return this.prisma.bountyApplication.findMany({ where: { bountyId } });
  }

  async reviewApplication(
    bountyId: string,
    appId: string,
    accept: boolean,
    reviewerId: string,
    reviewMessage?: string,
  ) {
    const bounty = await this.prisma.bounty.findUnique({
      where: { id: bountyId },
    });
    if (!bounty) throw new NotFoundException('Bounty not found');
    if (bounty.creatorId !== reviewerId)
      throw new ForbiddenException('Not allowed');

    const app = await this.prisma.bountyApplication.findUnique({
      where: { id: appId },
    });
    if (!app || app.bountyId !== bountyId)
      throw new NotFoundException('Application not found');

    const status = accept ? 'ACCEPTED' : 'REJECTED';
    const updated = await this.prisma.bountyApplication.update({
      where: { id: appId },
      data: { status, reviewerId, reviewMessage: reviewMessage || null },
    });

    if (accept) {
      // assign bounty
      await this.prisma.bounty.update({
        where: { id: bountyId },
        data: { assigneeId: app.applicantId, status: 'IN_PROGRESS' },
      });
    }

    return updated;
  }

  async createMilestone(
    bountyId: string,
    dto: CreateMilestoneDto,
    userId: string,
  ) {
    const bounty = await this.prisma.bounty.findUnique({
      where: { id: bountyId },
    });
    if (!bounty) throw new NotFoundException('Bounty not found');
    if (bounty.creatorId !== userId)
      throw new ForbiddenException('Not allowed');
    const data: any = {
      bountyId,
      title: dto.title,
      description: dto.description || null,
      amount: dto.amount,
    };
    if (dto.dueDate) data.dueDate = new Date(dto.dueDate);
    return this.prisma.bountyMilestone.create({ data });
  }

  async completeMilestone(
    bountyId: string,
    milestoneId: string,
    userId: string,
  ) {
    const milestone = await this.prisma.bountyMilestone.findUnique({
      where: { id: milestoneId },
    });
    if (!milestone || milestone.bountyId !== bountyId)
      throw new NotFoundException('Milestone not found');
    const bounty = await this.prisma.bounty.findUnique({
      where: { id: bountyId },
    });
    if (!bounty) throw new NotFoundException('Bounty not found');
    if (bounty.assigneeId !== userId)
      throw new ForbiddenException('Only assignee can mark complete');
    return this.prisma.bountyMilestone.update({
      where: { id: milestoneId },
      data: { status: 'COMPLETE', completedAt: new Date() },
    });
  }

  async approveMilestone(
    bountyId: string,
    milestoneId: string,
    userId: string,
  ) {
    const milestone = await this.prisma.bountyMilestone.findUnique({
      where: { id: milestoneId },
    });
    if (!milestone || milestone.bountyId !== bountyId)
      throw new NotFoundException('Milestone not found');
    const bounty = await this.prisma.bounty.findUnique({
      where: { id: bountyId },
    });
    if (!bounty) throw new NotFoundException('Bounty not found');
    if (bounty.creatorId !== userId)
      throw new ForbiddenException('Only creator can approve milestone');
    if (milestone.status !== 'COMPLETE')
      throw new BadRequestException('Milestone not completed');

    // create payout record
    const payout = await this.prisma.bountyPayout.create({
      data: {
        bountyId,
        toUserId: bounty.assigneeId as string,
        amount: milestone.amount,
        token: bounty.rewardToken || 'STELLAR',
        status: 'SENT',
        processedAt: new Date(),
      },
    });

    // mark milestone approved
    await this.prisma.bountyMilestone.update({
      where: { id: milestoneId },
      data: { status: 'APPROVED' },
    });

    // notify assignee
    try {
      const assignee = await this.prisma.user.findUnique({
        where: { id: bounty.assigneeId as string },
      });
      if (assignee?.email)
        await this.mailer.sendRevokeEmail(
          assignee.email,
          `Payout processed for milestone ${milestone.title}`,
          undefined,
        );
    } catch (_) {}

    return { payout, milestoneApproved: true };
  }
}
