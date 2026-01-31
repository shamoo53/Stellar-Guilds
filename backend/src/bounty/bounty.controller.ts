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
import { BountyService } from './bounty.service';
import { CreateBountyDto } from './dto/create-bounty.dto';
import { UpdateBountyDto } from './dto/update-bounty.dto';
import { ApplyBountyDto } from './dto/apply-bounty.dto';
import { CreateMilestoneDto } from './dto/create-milestone.dto';

@Controller('bounties')
export class BountyController {
  constructor(private service: BountyService) {}

  @UseGuards(JwtAuthGuard)
  @Post()
  async create(@Body() dto: CreateBountyDto, @Request() req: any) {
    return this.service.create(dto, req.user.userId);
  }

  @Get(':id')
  async get(@Param('id') id: string) {
    return this.service.get(id);
  }

  @Get()
  async search(
    @Query('q') q: string,
    @Query('page') page = '0',
    @Query('size') size = '20',
    @Query('guildId') guildId?: string,
  ) {
    return this.service.search(q, Number(page), Number(size), guildId);
  }

  @UseGuards(JwtAuthGuard)
  @Patch(':id')
  async update(
    @Param('id') id: string,
    @Body() dto: UpdateBountyDto,
    @Request() req: any,
  ) {
    return this.service.update(id, dto, req.user.userId);
  }

  @UseGuards(JwtAuthGuard)
  @Post(':id/cancel')
  async cancel(@Param('id') id: string, @Request() req: any) {
    return this.service.cancel(id, req.user.userId);
  }

  @UseGuards(JwtAuthGuard)
  @Post(':id/apply')
  async apply(
    @Param('id') id: string,
    @Body() dto: ApplyBountyDto,
    @Request() req: any,
  ) {
    return this.service.apply(id, dto, req.user.userId);
  }

  @UseGuards(JwtAuthGuard)
  @Get(':id/applications')
  async listApplications(@Param('id') id: string, @Request() req: any) {
    return this.service.listApplications(id, req.user.userId);
  }

  @UseGuards(JwtAuthGuard)
  @Post(':id/applications/:appId/review')
  async reviewApplication(
    @Param('id') id: string,
    @Param('appId') appId: string,
    @Body() body: any,
    @Request() req: any,
  ) {
    const accept = !!body.accept;
    return this.service.reviewApplication(
      id,
      appId,
      accept,
      req.user.userId,
      body.message,
    );
  }

  @UseGuards(JwtAuthGuard)
  @Post(':id/milestones')
  async createMilestone(
    @Param('id') id: string,
    @Body() dto: CreateMilestoneDto,
    @Request() req: any,
  ) {
    return this.service.createMilestone(id, dto, req.user.userId);
  }

  @UseGuards(JwtAuthGuard)
  @Post(':id/milestones/:mid/complete')
  async completeMilestone(
    @Param('id') id: string,
    @Param('mid') mid: string,
    @Request() req: any,
  ) {
    return this.service.completeMilestone(id, mid, req.user.userId);
  }

  @UseGuards(JwtAuthGuard)
  @Post(':id/milestones/:mid/approve')
  async approveMilestone(
    @Param('id') id: string,
    @Param('mid') mid: string,
    @Request() req: any,
  ) {
    return this.service.approveMilestone(id, mid, req.user.userId);
  }
}
