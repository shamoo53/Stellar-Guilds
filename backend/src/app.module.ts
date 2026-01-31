import { Module } from '@nestjs/common';
import { AppController } from './app.controller';
import { AppService } from './app.service';
import { ConfigModule } from '@nestjs/config';
import { PrismaModule } from './prisma/prisma.module';
import { UserModule } from './user/user.module';
import { AuthModule } from './auth/auth.module';
import { GuildModule } from './guild/guild.module';
import { BountyModule } from './bounty/bounty.module';

@Module({
  imports: [
    ConfigModule.forRoot({
      isGlobal: true,
    }),
    PrismaModule,
    AuthModule,
    UserModule,
    GuildModule,
    BountyModule,
  ],
  controllers: [AppController],
  providers: [AppService],
})
export class AppModule {}
