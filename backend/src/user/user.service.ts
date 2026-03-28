import {
  Injectable,
  NotFoundException,
  BadRequestException,
  ForbiddenException,
  Logger,
} from '@nestjs/common';
import { PrismaService } from '../prisma/prisma.service';
import { StorageService } from '../storage/storage.service';
import {
  UpdateUserDto,
  ChangePasswordDto,
  SearchUserDto,
  AssignRoleDto,
  UserRole,
} from './dto/user.dto';
import * as bcrypt from 'bcrypt';

@Injectable()
export class UserService {
  private readonly logger = new Logger(UserService.name);

  constructor(
    private prisma: PrismaService,
    private storageService: StorageService,
  ) {}

  /**
   * Get user by ID with public profile information
   */
  async getUserProfile(userId: string) {
    const user = await this.prisma.user.findUnique({
      where: { id: userId },
      select: {
        id: true,
        username: true,
        firstName: true,
        lastName: true,
        bio: true,
        location: true,
        avatarUrl: true,
        profileBio: true,
        profileUrl: true,
        discordHandle: true,
        twitterHandle: true,
        githubHandle: true,
        createdAt: true,
        role: true,
      },
    });

    if (!user) {
      throw new NotFoundException('User not found');
    }

    return user;
  }

  /**
   * Get user details (sensitive info - admin/self only)
   */
  async getUserDetails(
    userId: string,
    requesterId: string,
    requesterRole: UserRole,
  ) {
    if (userId !== requesterId && requesterRole !== UserRole.ADMIN) {
      throw new ForbiddenException(
        'You do not have permission to view this user details',
      );
    }

    const user = await this.prisma.user.findUnique({
      where: { id: userId },
      select: {
        id: true,
        email: true,
        username: true,
        firstName: true,
        lastName: true,
        bio: true,
        location: true,
        avatarUrl: true,
        profileBio: true,
        profileUrl: true,
        discordHandle: true,
        twitterHandle: true,
        githubHandle: true,
        walletAddress: true,
        role: true,
        isActive: true,
        lastLoginAt: true,
        createdAt: true,
        updatedAt: true,
      },
    });

    if (!user) {
      throw new NotFoundException('User not found');
    }

    return user;
  }

  /**
   * Update user profile
   */
  async updateUserProfile(userId: string, updateDto: UpdateUserDto) {
    const user = await this.prisma.user.findUnique({
      where: { id: userId },
    });

    if (!user) {
      throw new NotFoundException('User not found');
    }

    const updated = await this.prisma.user.update({
      where: { id: userId },
      data: {
        ...(updateDto.firstName && { firstName: updateDto.firstName }),
        ...(updateDto.lastName && { lastName: updateDto.lastName }),
        ...(updateDto.bio !== undefined && { bio: updateDto.bio }),
        ...(updateDto.location !== undefined && { location: updateDto.location }),
        ...(updateDto.profileBio !== undefined && {
          profileBio: updateDto.profileBio,
        }),
        ...(updateDto.profileUrl !== undefined && {
          profileUrl: updateDto.profileUrl,
        }),
        ...(updateDto.discordHandle !== undefined && {
          discordHandle: updateDto.discordHandle,
        }),
        ...(updateDto.twitterHandle !== undefined && {
          twitterHandle: updateDto.twitterHandle,
        }),
        ...(updateDto.githubHandle !== undefined && {
          githubHandle: updateDto.githubHandle,
        }),
      },
      select: {
        id: true,
        username: true,
        firstName: true,
        lastName: true,
        bio: true,
        location: true,
        avatarUrl: true,
        profileBio: true,
        profileUrl: true,
        discordHandle: true,
        twitterHandle: true,
        githubHandle: true,
        createdAt: true,
        updatedAt: true,
        role: true,
      },
    });

    return updated;
  }

  /**
   * Change user password
   */
  async changePassword(userId: string, changePasswordDto: ChangePasswordDto) {
    const { currentPassword, newPassword, confirmPassword } = changePasswordDto;

    // Validate password confirmation
    if (newPassword !== confirmPassword) {
      throw new BadRequestException('Passwords do not match');
    }

    // Prevent same password
    if (currentPassword === newPassword) {
      throw new BadRequestException(
        'New password must be different from current password',
      );
    }

    const user = await this.prisma.user.findUnique({
      where: { id: userId },
    });

    if (!user) {
      throw new NotFoundException('User not found');
    }

    // Verify current password
    const isPasswordValid = await bcrypt.compare(
      currentPassword,
      user.password,
    );
    if (!isPasswordValid) {
      throw new BadRequestException('Current password is incorrect');
    }

    // Hash and update new password
    const hashedPassword = await bcrypt.hash(newPassword, 10);
    await this.prisma.user.update({
      where: { id: userId },
      data: { password: hashedPassword },
    });

    return { message: 'Password changed successfully' };
  }

  /**
   * Update user avatar URL
   */
  async updateAvatar(
    userId: string,
    file: { buffer: Buffer; originalname: string },
  ) {
    const user = await this.prisma.user.findUnique({
      where: { id: userId },
    });

    if (!user) {
      throw new NotFoundException('User not found');
    }

    const avatarUrl = await this.storageService.uploadFile(
      file.buffer,
      file.originalname,
    );

    const updated = await this.prisma.user.update({
      where: { id: userId },
      data: { avatarUrl },
      select: {
        id: true,
        avatarUrl: true,
      },
    });

    if (user.avatarUrl) {
      try {
        await this.storageService.deleteFile(user.avatarUrl);
      } catch (error: any) {
        this.logger.warn(
          `Failed to delete previous avatar for user ${userId}: ${error?.message ?? 'unknown error'}`,
        );
      }
    }

    return updated;
  }

  /**
   * Search and filter users (paginated)
   */
  async searchUsers(searchDto: SearchUserDto) {
    const { query, role, isActive, skip = 0, take = 20 } = searchDto;

    // Build where clause
    const where: any = {};

    if (query) {
      where.OR = [
        { username: { contains: query, mode: 'insensitive' } },
        { email: { contains: query, mode: 'insensitive' } },
        { firstName: { contains: query, mode: 'insensitive' } },
        { lastName: { contains: query, mode: 'insensitive' } },
      ];
    }

    if (role) {
      where.role = role;
    }

    if (isActive !== undefined) {
      where.isActive = isActive;
    }

    const [users, total] = await Promise.all([
      this.prisma.user.findMany({
        where,
        select: {
          id: true,
          username: true,
          firstName: true,
          lastName: true,
          bio: true,
          location: true,
          avatarUrl: true,
          profileBio: true,
          profileUrl: true,
          discordHandle: true,
          twitterHandle: true,
          githubHandle: true,
          createdAt: true,
          role: true,
        },
        skip,
        take,
        orderBy: { createdAt: 'desc' },
      }),
      this.prisma.user.count({ where }),
    ]);

    return {
      data: users,
      total,
      skip,
      take,
    };
  }

  /**
   * Get users by role
   */
  async getUsersByRole(role: UserRole, skip = 0, take = 20) {
    const users = await this.prisma.user.findMany({
      where: { role },
      select: {
        id: true,
        username: true,
        firstName: true,
        lastName: true,
        bio: true,
        location: true,
        avatarUrl: true,
        createdAt: true,
        role: true,
      },
      skip,
      take,
      orderBy: { createdAt: 'desc' },
    });

    const total = await this.prisma.user.count({ where: { role } });

    return {
      data: users,
      total,
      skip,
      take,
    };
  }

  /**
   * Assign role to user (admin only)
   */
  async assignRole(userId: string, assignRoleDto: AssignRoleDto) {
    const user = await this.prisma.user.findUnique({
      where: { id: userId },
    });

    if (!user) {
      throw new NotFoundException('User not found');
    }

    const updated = await this.prisma.user.update({
      where: { id: userId },
      data: { role: assignRoleDto.role },
      select: {
        id: true,
        username: true,
        email: true,
        role: true,
      },
    });

    return updated;
  }

  /**
   * Deactivate user account
   */
  async deactivateUser(userId: string) {
    const user = await this.prisma.user.findUnique({
      where: { id: userId },
    });

    if (!user) {
      throw new NotFoundException('User not found');
    }

    await this.prisma.user.update({
      where: { id: userId },
      data: { isActive: false },
    });

    return { message: 'User account deactivated successfully' };
  }

  /**
   * Reactivate user account
   */
  async reactivateUser(userId: string) {
    const user = await this.prisma.user.findUnique({
      where: { id: userId },
    });

    if (!user) {
      throw new NotFoundException('User not found');
    }

    await this.prisma.user.update({
      where: { id: userId },
      data: { isActive: true },
    });

    return { message: 'User account reactivated successfully' };
  }

  // Existing basic CRUD methods (kept for compatibility)
  async user(userWhereUniqueInput: any): Promise<any | null> {
    return this.prisma.user.findUnique({
      where: userWhereUniqueInput,
    });
  }

  async users(params: {
    skip?: number;
    take?: number;
    cursor?: any;
    where?: any;
    orderBy?: any;
  }): Promise<any[]> {
    const { skip, take, cursor, where, orderBy } = params;
    return this.prisma.user.findMany({
      skip,
      take,
      cursor,
      where,
      orderBy,
    });
  }

  async createUser(data: any): Promise<any> {
    return this.prisma.user.create({
      data,
    });
  }

  async updateUser(params: { where: any; data: any }): Promise<any> {
    const { where, data } = params;
    return this.prisma.user.update({
      data,
      where,
    });
  }

  async deleteUser(where: any): Promise<any> {
    return this.prisma.user.delete({
      where,
    });
  }
}
