import {
  Controller,
  Get,
  Post,
  Patch,
  Delete,
  Body,
  Param,
  Query,
  UseGuards,
  Request,
  HttpCode,
  HttpStatus,
  BadRequestException,
  UploadedFile,
  UseInterceptors,
} from '@nestjs/common';
import { FileInterceptor } from '@nestjs/platform-express';
import {
  ApiTags,
  ApiOperation,
  ApiResponse,
  ApiBearerAuth,
  ApiParam,
} from '@nestjs/swagger';
import { UserService } from './user.service';
import { JwtAuthGuard } from '../auth/guards/jwt-auth.guard';
import { RoleGuard } from '../auth/guards/role.guard';
import { Roles } from '../auth/decorators/roles.decorator';
import {
  UpdateUserProfileDto,
  ChangePasswordDto,
  SearchUserDto,
  AssignRoleDto,
  UserRole,
  UserProfileDto,
} from './dto/user.dto';

@Controller('users')
export class UserController {
  constructor(private userService: UserService) {}

  /**
   * Get current authenticated user profile
   */
  @Get('me')
  @UseGuards(JwtAuthGuard)
  @HttpCode(HttpStatus.OK)
  async getCurrentUser(@Request() req: any) {
    return this.userService.getUserProfile(req.user.userId);
  }

  /**
   * Search and filter users (must come before :userId to avoid route conflicts)
   */
  @Get('search')
  @HttpCode(HttpStatus.OK)
  async searchUsers(@Query() searchDto: SearchUserDto) {
    return this.userService.searchUsers(searchDto);
  }

  /**
   * Get user profile by ID (public)
   * Returns user profile excluding sensitive fields like password, email, walletAddress
   */
  @Get(':userId')
  @ApiOperation({ summary: 'Get user profile by ID (public)' })
  @ApiParam({ name: 'userId', description: 'User ID (UUID)' })
  @ApiResponse({
    status: HttpStatus.OK,
    description: 'User profile retrieved successfully',
    type: UserProfileDto,
  })
  @ApiResponse({
    status: HttpStatus.NOT_FOUND,
    description: 'User not found',
  })
  @HttpCode(HttpStatus.OK)
  async getUserProfile(@Param('userId') userId: string) {
    return this.userService.getUserProfile(userId);
  }

  /**
   * Update current user profile
   */
  @Patch('me')
  @UseGuards(JwtAuthGuard)
  @HttpCode(HttpStatus.OK)
  async updateProfile(
    @Request() req: any,
    @Body() updateDto: UpdateUserProfileDto,
  ) {
    return this.userService.updateUserProfile(req.user.userId, updateDto);
  }

  /**
   * Change current user password
   */
  @Post('me/change-password')
  @UseGuards(JwtAuthGuard)
  @HttpCode(HttpStatus.OK)
  async changePassword(
    @Request() req: any,
    @Body() changePasswordDto: ChangePasswordDto,
  ) {
    return this.userService.changePassword(req.user.userId, changePasswordDto);
  }

  /**
   * Upload user avatar
   * Accepts multipart/form-data with a single "file" field.
   */
  @Post('me/avatar')
  @UseGuards(JwtAuthGuard)
  @UseInterceptors(FileInterceptor('file'))
  @HttpCode(HttpStatus.OK)
  async uploadAvatar(
    @Request() req: any,
    @UploadedFile() file: any,
  ) {
    if (!file?.buffer || !file?.originalname) {
      throw new BadRequestException('file is required');
    }
    const result = await this.userService.updateAvatar(req.user.userId, file);
    return {
      avatarUrl: result.avatarUrl,
      message: 'Avatar updated successfully',
    };
  }

  /**
   * Deactivate current user account
   */
  @Delete('me')
  @UseGuards(JwtAuthGuard)
  @HttpCode(HttpStatus.OK)
  async deactivateAccount(@Request() req: any) {
    return this.userService.deactivateUser(req.user.userId);
  }

  /**
   * Get user details (admin or self)
   */
  @Get('details/:userId')
  @UseGuards(JwtAuthGuard)
  @HttpCode(HttpStatus.OK)
  async getUserDetails(@Param('userId') userId: string, @Request() req: any) {
    return this.userService.getUserDetails(
      userId,
      req.user.userId,
      req.user.role,
    );
  }

  /**
   * Admin: Get users by role
   */
  @Get('role/:role')
  @UseGuards(JwtAuthGuard, RoleGuard)
  @Roles(UserRole.ADMIN, UserRole.MODERATOR)
  @HttpCode(HttpStatus.OK)
  async getUsersByRole(
    @Param('role') role: UserRole,
    @Query('skip') skip?: string,
    @Query('take') take?: string,
  ) {
    return this.userService.getUsersByRole(
      role,
      skip ? parseInt(skip) : 0,
      take ? parseInt(take) : 20,
    );
  }

  /**
   * Admin: Assign role to user
   */
  @Patch(':userId/role')
  @UseGuards(JwtAuthGuard, RoleGuard)
  @Roles(UserRole.ADMIN)
  @HttpCode(HttpStatus.OK)
  async assignRole(
    @Param('userId') userId: string,
    @Body() assignRoleDto: AssignRoleDto,
  ) {
    return this.userService.assignRole(userId, assignRoleDto);
  }

  /**
   * Admin: Reactivate user account
   */
  @Post(':userId/reactivate')
  @UseGuards(JwtAuthGuard, RoleGuard)
  @Roles(UserRole.ADMIN)
  @HttpCode(HttpStatus.OK)
  async reactivateUser(@Param('userId') userId: string) {
    return this.userService.reactivateUser(userId);
  }
}
