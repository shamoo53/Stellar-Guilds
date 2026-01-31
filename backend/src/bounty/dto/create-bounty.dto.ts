import {
  IsString,
  IsOptional,
  IsNumber,
  IsDecimal,
  IsPositive,
  MaxLength,
  IsISO8601,
} from 'class-validator';

export class CreateBountyDto {
  @IsString()
  @MaxLength(200)
  title!: string;

  @IsString()
  @MaxLength(5000)
  description!: string;

  @IsOptional()
  @IsNumber()
  rewardAmount?: number;

  @IsOptional()
  @IsString()
  rewardToken?: string;

  @IsOptional()
  @IsISO8601()
  deadline?: string;

  @IsOptional()
  @IsString()
  guildId?: string;
}
