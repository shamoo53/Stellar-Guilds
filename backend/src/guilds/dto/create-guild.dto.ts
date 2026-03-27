import { ApiProperty } from '@nestjs/swagger';

export class CreateGuildDto {
  @ApiProperty({
    example: 'My Guild',
    description: 'Name of the guild',
  })
  name: string;
}