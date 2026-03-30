import { IsOptional, IsString, IsNumber, Min, IsIn } from 'class-validator';
import { Type } from 'class-transformer';

export class FindBountyDto {
  @IsOptional()
  @IsString()
  @IsIn(['OPEN', 'IN_PROGRESS', 'IN_REVIEW', 'SUBMITTED_FOR_REVIEW', 'COMPLETED_PENDING_CLAIM', 'COMPLETED', 'CANCELLED'])
  status?: string;

  @IsOptional()
  @IsString()
  tokenType?: string;

  @IsOptional()
  @Type(() => Number)
  @IsNumber()
  @Min(0)
  minReward?: number;

  @IsOptional()
  @IsString()
  guildId?: string;
}
