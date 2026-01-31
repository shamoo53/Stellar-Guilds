import {
  Injectable,
  OnModuleInit,
  OnModuleDestroy,
  Logger,
} from '@nestjs/common';
import { PrismaClient } from '@prisma/client';

@Injectable()
export class PrismaService implements OnModuleInit, OnModuleDestroy {
  private readonly logger = new Logger(PrismaService.name);
  private prisma: PrismaClient;

  constructor() {
    try {
      // Create PrismaClient with minimal options
      const options: any = {};
      this.prisma = new PrismaClient(options);
    } catch (error) {
      this.logger.error(`Failed to create PrismaClient: ${error}`);
      throw error;
    }
  }

  async onModuleInit() {
    try {
      await this.prisma.$connect();
      this.logger.log('✅ Database connected successfully');
    } catch (error) {
      this.logger.warn(`⚠️ Database connection failed (will retry): ${error}`);
      // Continue - database may not be running yet
    }
  }

  async onModuleDestroy() {
    try {
      await this.prisma.$disconnect();
    } catch (error) {
      this.logger.warn(`Error disconnecting: ${error}`);
    }
  }

  // Expose Prisma models
  get user() {
    return this.prisma.user;
  }

  get guild() {
    return this.prisma.guild;
  }

  get guildMembership() {
    return this.prisma.guildMembership;
  }

  get bounty() {
    return this.prisma.bounty;
  }

  get bountyApplication() {
    return this.prisma.bountyApplication;
  }

  get bountyMilestone() {
    return this.prisma.bountyMilestone;
  }

  get bountyPayout() {
    return this.prisma.bountyPayout;
  }

  get role() {
    return this.prisma.role;
  }

  get permission() {
    return this.prisma.permission;
  }

  // Expose Prisma utilities
  get $transaction() {
    return this.prisma.$transaction.bind(this.prisma);
  }

  get $queryRaw() {
    return this.prisma.$queryRaw.bind(this.prisma);
  }

  get $executeRaw() {
    return this.prisma.$executeRaw.bind(this.prisma);
  }

  get $connect() {
    return this.prisma.$connect.bind(this.prisma);
  }

  get $disconnect() {
    return this.prisma.$disconnect.bind(this.prisma);
  }
}
