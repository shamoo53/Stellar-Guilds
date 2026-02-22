import { Controller, Get, Param } from '@nestjs/common';
import { SocialService } from './social.service';

@Controller('social')
export class SocialController {
  constructor(private readonly socialService: SocialService) {}

  @Get('feed/:userId')
  async getFeed(@Param('userId') userId: string) {
    return this.socialService.getFeedForUser(userId);
  }

  // additional endpoints (notifications, messages, forums) will be added here
}
