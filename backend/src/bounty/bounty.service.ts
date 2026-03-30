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
import { ReviewWorkDto } from './dto/review-work.dto';
import { SubmitBountyWorkDto } from './dto/submit-work.dto';

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

  async findOne(id: string) {
    const bounty = await this.prisma.bounty.findUnique({
      where: { id },
      include: {
        creator: true,
        assignee: true,
      },
    });
    if (!bounty) throw new NotFoundException('Bounty not found');
    return bounty;
  }

  async get(id: string) {
    const bounty = await this.prisma.bounty.findUnique({
      where: { id },
      include: { creator: true, assignee: true },
    });
    if (!bounty) throw new NotFoundException('Bounty not found');
    return bounty;
  }

  async findAll(filters: {
    page?: number;
    size?: number;
    status?: string;
    tokenType?: string;
    minReward?: number;
    guildId?: string;
  }) {
    const page = filters.page ?? 0;
    const size = filters.size ?? 20;

    const where: any = {};

    // Default to OPEN status if no status filter provided
    if (filters.status) {
      where.status = filters.status;
    } else {
      where.status = 'OPEN';
    }

    if (filters.tokenType) {
      where.rewardToken = filters.tokenType;
    }

    if (filters.minReward !== undefined) {
      where.rewardAmount = { gte: filters.minReward };
    }

    if (filters.guildId) {
      where.guildId = filters.guildId;
    }

    const [items, total] = await Promise.all([
      this.prisma.bounty.findMany({
        where,
        skip: page * size,
        take: size,
        orderBy: { createdAt: 'desc' },
      }),
      this.prisma.bounty.count({ where }),
    ]);
    return { items, total, page, size };
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

  /**
   * Submit work for review
   * State machine transition: IN_PROGRESS -> IN_REVIEW
   */
  async submitWork(
    bountyId: string,
    dto: SubmitBountyWorkDto,
    userId: string,
  ) {
    const bounty = await this.prisma.bounty.findUnique({
      where: { id: bountyId },
      include: { creator: true },
    });

    if (!bounty) {
      throw new NotFoundException('Bounty not found');
    }

    // Only assignee can submit work
    if (bounty.assigneeId !== userId) {
      throw new ForbiddenException('Only the assigned user can submit work');
    }

    // Validate current status allows submission
    if (bounty.status !== 'IN_PROGRESS') {
      throw new BadRequestException(
        `Cannot submit work for bounty in ${bounty.status} status. Work can only be submitted when bounty is IN_PROGRESS.`,
      );
    }

    const updatedBounty = await this.prisma.$transaction(async (tx) => {
      const updateResult = await tx.bounty.updateMany({
        where: {
          id: bountyId,
          assigneeId: userId,
          status: 'IN_PROGRESS',
        },
        data: {
          status: 'IN_REVIEW',
        },
      });

      if (updateResult.count !== 1) {
        const latestBounty = await tx.bounty.findUnique({
          where: { id: bountyId },
        });

        if (!latestBounty) {
          throw new NotFoundException('Bounty not found');
        }

        if (latestBounty.assigneeId !== userId) {
          throw new ForbiddenException(
            'Only the assigned user can submit work',
          );
        }

        throw new BadRequestException(
          `Cannot submit work for bounty in ${latestBounty.status} status. Work can only be submitted when bounty is IN_PROGRESS.`,
        );
      }

      return tx.bounty.findUnique({
        where: { id: bountyId },
      });
    // Build submission data from DTO
    const submissionData = {
      submissions: dto.submissions.map((sub) => ({
        prUrl: sub.prUrl,
        description: sub.description,
      })),
      attachmentUrls: dto.attachmentUrls || [],
      additionalComments: dto.additionalComments || null,
      submittedAt: new Date(),
    };

    const updatedBounty = await this.prisma.bounty.update({
      where: { id: bountyId },
      data: {
        status: 'SUBMITTED_FOR_REVIEW',
      },
    });

    // Store submission details in metadata or a separate table
    // For now, we'll create a notification with the submission data
    try {
      await this.prisma.notification.create({
        data: {
          userId: bounty.creatorId,
          message: `Work submitted for "${bounty.title}"`,
          type: 'BOUNTY_WORK_SUBMITTED',
          metadata: submissionData,
        },
      });
    } catch (_) {
      // Ignore notification errors
    }

    // Notify bounty creator of submission
    try {
      if (bounty.creator?.email) {
        await this.mailer.sendRevokeEmail(
          bounty.creator.email,
          `Work submitted for "${bounty.title}"`,
          `The assigned user has submitted work for review. Check the platform for details.`,
        );
      }
    } catch (_) {
      // Ignore email errors
    }

    return {
      bounty: updatedBounty,
      submission: submissionData,
      message: 'Work submitted successfully. Awaiting review.',
    };
  }

  /**
   * Review submitted bounty work
   * State machine transitions:
   * - IN_REVIEW + approve -> COMPLETED_PENDING_CLAIM
   * - IN_REVIEW + reject -> IN_PROGRESS (with feedback)
   */
  async reviewWork(
    bountyId: string,
    dto: ReviewWorkDto,
    reviewerId: string,
  ) {
    const bounty = await this.prisma.bounty.findUnique({
      where: { id: bountyId },
      include: { assignee: true },
    });

    if (!bounty) {
      throw new NotFoundException('Bounty not found');
    }

    // Only creator or guild admin can review work
    if (bounty.creatorId !== reviewerId) {
      throw new ForbiddenException('Only the bounty creator can review work');
    }

    // Validate current status allows review
    if (
      bounty.status !== 'IN_REVIEW' &&
      bounty.status !== 'SUBMITTED_FOR_REVIEW'
    ) {
      throw new BadRequestException(
        `Cannot review bounty in ${bounty.status} status. Work must be in review first.`,
      );
    }

    if (dto.approve) {
      // Approve: transition to COMPLETED_PENDING_CLAIM
      const updatedBounty = await this.prisma.bounty.update({
        where: { id: bountyId },
        data: {
          status: 'COMPLETED_PENDING_CLAIM',
        },
      });

      // Notify assignee of approval
      try {
        if (bounty.assignee?.email) {
          await this.mailer.sendRevokeEmail(
            bounty.assignee.email,
            `Your work on "${bounty.title}" has been approved!`,
            `You can now claim your reward of ${bounty.rewardAmount} ${bounty.rewardToken}.`,
          );
        }
      } catch (_) {
        // Ignore email errors
      }

      return {
        bounty: updatedBounty,
        action: 'APPROVED',
        message: 'Work approved. Bounty is now ready for reward claim.',
      };
    } else {
      // Reject: transition back to IN_PROGRESS with feedback
      if (!dto.feedback || dto.feedback.trim().length === 0) {
        throw new BadRequestException(
          'Rejection feedback is required when rejecting work',
        );
      }

      const updatedBounty = await this.prisma.bounty.update({
        where: { id: bountyId },
        data: {
          status: 'IN_PROGRESS',
        },
      });

      // Store rejection feedback as a notification or in a separate table
      // For now, we'll create a notification for the assignee
      try {
        if (bounty.assigneeId) {
          await this.prisma.notification.create({
            data: {
              userId: bounty.assigneeId,
              message: `Your work on "${bounty.title}" was rejected. Feedback: ${dto.feedback}`,
              type: 'BOUNTY_WORK_REJECTED',
              metadata: {
                bountyId,
                feedback: dto.feedback,
              },
            },
          });
        }

        if (bounty.assignee?.email) {
          await this.mailer.sendRevokeEmail(
            bounty.assignee.email,
            `Your work on "${bounty.title}" needs revision`,
            `Feedback: ${dto.feedback}`,
          );
        }
      } catch (_) {
        // Ignore notification/email errors
      }

      return {
        bounty: updatedBounty,
        action: 'REJECTED',
        message: 'Work rejected. Bounty returned to IN_PROGRESS status.',
        feedback: dto.feedback,
      };
    }
  }
}
