import { Controller, Get, Post, Body, Param } from '@nestjs/common';
import {
  ApiTags,
  ApiOperation,
  ApiResponse,
  ApiParam,
} from '@nestjs/swagger';

import { GuildService } from './guild.service';
import { CreateGuildDto } from './dto/create-guild.dto';

@ApiTags('Guilds')
@Controller('guilds')
export class GuildController {
  constructor(private readonly guildService: GuildService) {}

  @Post()
  @ApiOperation({ summary: 'Create a new guild' })
  @ApiResponse({ status: 201, description: 'Guild created successfully' })
  create(@Body() dto: CreateGuildDto) {
    return this.guildService.createGuild(dto);
  }

  @Get(':slug')
  @ApiOperation({ summary: 'Get guild by slug' })
  @ApiParam({ name: 'slug', example: 'my-guild' })
  @ApiResponse({ status: 200, description: 'Guild found' })
  getBySlug(@Param('slug') slug: string) {
    return this.guildService.getGuildBySlug(slug);
  }
}