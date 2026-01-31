import {
  IsString,
  IsNumber,
  MaxLength,
  IsOptional,
  IsISO8601,
} from 'class-validator';

export class CreateMilestoneDto {
  @IsString()
  @MaxLength(200)
  title!: string;

  @IsOptional()
  @IsString()
  @MaxLength(2000)
  description?: string;

  @IsNumber()
  amount!: number;

  @IsOptional()
  @IsISO8601()
  dueDate?: string;
}
