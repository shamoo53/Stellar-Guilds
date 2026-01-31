import {
  IsOptional,
  IsString,
  MaxLength,
  IsNumber,
  IsISO8601,
} from 'class-validator';

export class UpdateBountyDto {
  @IsOptional()
  @IsString()
  @MaxLength(200)
  title?: string;

  @IsOptional()
  @IsString()
  @MaxLength(5000)
  description?: string;

  @IsOptional()
  @IsNumber()
  rewardAmount?: number;

  @IsOptional()
  @IsString()
  rewardToken?: string;

  @IsOptional()
  @IsISO8601()
  deadline?: string;
}
