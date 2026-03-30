import { Test, TestingModule } from '@nestjs/testing';
import { ForbiddenException } from '@nestjs/common';
import { BountyService } from './bounty.service';
import { PrismaService } from '../prisma/prisma.service';
import { MailerService } from '../mailer/mailer.service';

const mockPrisma = () => {
  const prisma = {
    bounty: {
      findUnique: jest.fn(),
      findMany: jest.fn(),
      count: jest.fn(),
      updateMany: jest.fn(),
    },
    $transaction: jest.fn(),
  };

  prisma.$transaction.mockImplementation(async (callback) => callback(prisma));

  return prisma;
};

const mockMailer = () => ({
  sendInviteEmail: jest.fn(),
  sendRevokeEmail: jest.fn(),
});

describe('BountyService', () => {
  let service: BountyService;
  let prisma: ReturnType<typeof mockPrisma>;
  let mailer: ReturnType<typeof mockMailer>;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        BountyService,
        { provide: PrismaService, useFactory: mockPrisma },
        { provide: MailerService, useFactory: mockMailer },
      ],
    }).compile();

    service = module.get(BountyService);
    prisma = module.get(PrismaService);
    mailer = module.get(MailerService);
  });

  describe('findAll', () => {
    it('filters by status when status is provided', async () => {
      const mockBounties = [
        { id: 'b1', title: 'Bounty 1', status: 'IN_PROGRESS', rewardAmount: 100, rewardToken: 'STELLAR' },
      ];
      prisma.bounty.findMany.mockResolvedValue(mockBounties);
      prisma.bounty.count.mockResolvedValue(1);

      const result = await service.findAll({ status: 'IN_PROGRESS' });

      expect(prisma.bounty.findMany).toHaveBeenCalledWith(
        expect.objectContaining({
          where: expect.objectContaining({ status: 'IN_PROGRESS' }),
        }),
      );
      expect(result.items).toEqual(mockBounties);
      expect(result.total).toBe(1);
    });

    it('filters by tokenType (rewardToken) when provided', async () => {
      const mockBounties = [
        { id: 'b1', title: 'XLM Bounty', status: 'OPEN', rewardAmount: 50, rewardToken: 'XLM' },
      ];
      prisma.bounty.findMany.mockResolvedValue(mockBounties);
      prisma.bounty.count.mockResolvedValue(1);

      const result = await service.findAll({ tokenType: 'XLM' });

      expect(prisma.bounty.findMany).toHaveBeenCalledWith(
        expect.objectContaining({
          where: expect.objectContaining({ rewardToken: 'XLM' }),
        }),
      );
      expect(result.items).toEqual(mockBounties);
    });

    it('filters by minimum reward amount', async () => {
      const mockBounties = [
        { id: 'b1', title: 'High Value', status: 'OPEN', rewardAmount: 500, rewardToken: 'STELLAR' },
      ];
      prisma.bounty.findMany.mockResolvedValue(mockBounties);
      prisma.bounty.count.mockResolvedValue(1);

      const result = await service.findAll({ minReward: 200 });

      expect(prisma.bounty.findMany).toHaveBeenCalledWith(
        expect.objectContaining({
          where: expect.objectContaining({ rewardAmount: { gte: 200 } }),
        }),
      );
      expect(result.items).toEqual(mockBounties);
    });

    it('combines all filters: status, tokenType, and minReward', async () => {
      const mockBounties = [
        { id: 'b1', title: 'Combined Filter Test', status: 'OPEN', rewardAmount: 300, rewardToken: 'XLM' },
      ];
      prisma.bounty.findMany.mockResolvedValue(mockBounties);
      prisma.bounty.count.mockResolvedValue(1);

      const result = await service.findAll({
        status: 'OPEN',
        tokenType: 'XLM',
        minReward: 200,
      });

      expect(prisma.bounty.findMany).toHaveBeenCalledWith(
        expect.objectContaining({
          where: expect.objectContaining({
            status: 'OPEN',
            rewardToken: 'XLM',
            rewardAmount: { gte: 200 },
          }),
        }),
      );
      expect(result.items).toEqual(mockBounties);
    });

    it('defaults to OPEN status when no status filter is provided', async () => {
      prisma.bounty.findMany.mockResolvedValue([]);
      prisma.bounty.count.mockResolvedValue(0);

      await service.findAll({});

      expect(prisma.bounty.findMany).toHaveBeenCalledWith(
        expect.objectContaining({
          where: expect.objectContaining({ status: 'OPEN' }),
        }),
      );
    });

    it('applies guildId filter in combination with other filters', async () => {
      const mockBounties = [
        { id: 'b1', title: 'Guild Bounty', status: 'OPEN', rewardAmount: 100, rewardToken: 'STELLAR', guildId: 'guild-1' },
      ];
      prisma.bounty.findMany.mockResolvedValue(mockBounties);
      prisma.bounty.count.mockResolvedValue(1);

      const result = await service.findAll({
        status: 'OPEN',
        guildId: 'guild-1',
        minReward: 50,
      });

      expect(prisma.bounty.findMany).toHaveBeenCalledWith(
        expect.objectContaining({
          where: expect.objectContaining({
            status: 'OPEN',
            guildId: 'guild-1',
            rewardAmount: { gte: 50 },
          }),
        }),
      );
      expect(result.items).toEqual(mockBounties);
    });

    it('returns paginated results with correct page and size', async () => {
      prisma.bounty.findMany.mockResolvedValue([]);
      prisma.bounty.count.mockResolvedValue(50);

      const result = await service.findAll({ page: 2, size: 10 });

      expect(prisma.bounty.findMany).toHaveBeenCalledWith(
        expect.objectContaining({
          skip: 20,
          take: 10,
        }),
      );
      expect(result.page).toBe(2);
      expect(result.size).toBe(10);
      expect(result.total).toBe(50);
    });
  });

  describe('submitWork', () => {
    it('transitions an assigned active bounty to IN_REVIEW', async () => {
      const bounty = {
        id: 'bounty-1',
        title: 'Build API endpoint',
        assigneeId: 'worker-1',
        creatorId: 'creator-1',
        creator: { email: 'creator@example.com' },
        status: 'IN_PROGRESS',
      };

      prisma.bounty.findUnique
        .mockResolvedValueOnce(bounty)
        .mockResolvedValueOnce({ ...bounty, status: 'IN_REVIEW' });
      prisma.bounty.updateMany.mockResolvedValue({ count: 1 });
      mailer.sendRevokeEmail.mockResolvedValue(undefined);

      const result = await service.submitWork(
        'bounty-1',
        'https://example.com/submission',
        'worker-1',
      );

      expect(prisma.$transaction).toHaveBeenCalled();
      expect(prisma.bounty.updateMany).toHaveBeenCalledWith({
        where: {
          id: 'bounty-1',
          assigneeId: 'worker-1',
          status: 'IN_PROGRESS',
        },
        data: {
          status: 'IN_REVIEW',
        },
      });
      expect(result.bounty.status).toBe('IN_REVIEW');
      expect(mailer.sendRevokeEmail).toHaveBeenCalled();
    });

    it('throws 403 when a non-assigned user submits work', async () => {
      prisma.bounty.findUnique.mockResolvedValue({
        id: 'bounty-1',
        title: 'Build API endpoint',
        assigneeId: 'worker-1',
        creatorId: 'creator-1',
        creator: { email: 'creator@example.com' },
        status: 'IN_PROGRESS',
      });

      await expect(
        service.submitWork(
          'bounty-1',
          'https://example.com/submission',
          'intruder-1',
        ),
      ).rejects.toThrow(ForbiddenException);

      expect(prisma.$transaction).not.toHaveBeenCalled();
      expect(prisma.bounty.updateMany).not.toHaveBeenCalled();
    });
  });
});
