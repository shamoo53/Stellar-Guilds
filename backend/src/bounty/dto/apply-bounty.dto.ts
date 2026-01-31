import { IsOptional, IsString, MaxLength } from 'class-validator';

export class ApplyBountyDto {
  @IsOptional()
  @IsString()
  @MaxLength(5000)
  message?: string;

  @IsOptional()
  attachments?: any;
}
