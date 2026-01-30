import { Injectable, Logger } from '@nestjs/common';
import * as nodemailer from 'nodemailer';

@Injectable()
export class MailerService {
  private transporter: nodemailer.Transporter | null = null;
  private readonly logger = new Logger(MailerService.name);

  constructor() {
    const host = process.env.SMTP_HOST;
    const port = process.env.SMTP_PORT ? Number(process.env.SMTP_PORT) : undefined;
    const user = process.env.SMTP_USER;
    const pass = process.env.SMTP_PASS;

    if (host && port) {
      this.transporter = nodemailer.createTransport({
        host,
        port,
        secure: port === 465,
        auth: user && pass ? { user, pass } : undefined,
      });
    } else {
      this.logger.warn('SMTP not configured — Mailer will log invites instead of sending');
    }
  }

  async sendInviteEmail(to: string, guildName: string, token: string, inviterEmail?: string) {
    const from = process.env.MAIL_FROM || process.env.SMTP_USER || 'no-reply@stellar-guilds.local';
    const inviteUrl = `${process.env.FRONTEND_URL || 'https://app.stellar-guilds.local'}/invites/accept?token=${token}`;
    const subject = `You are invited to join ${guildName}`;
    const text = `You have been invited to join ${guildName}.
Click here to accept: ${inviteUrl}

If you weren't expecting this invite, ignore this message.`;

    if (this.transporter) {
      await this.transporter.sendMail({ from, to, subject, text });
      this.logger.log(`Invite email sent to ${to}`);
    } else {
      // Fallback: log the invite details for manual delivery
      this.logger.log(`Invite (not sent) -> to: ${to}, subject: ${subject}, token: ${token}`);
    }
  }

  async sendRevokeEmail(to: string, guildName: string, inviterEmail?: string) {
    const from = process.env.MAIL_FROM || process.env.SMTP_USER || 'no-reply@stellar-guilds.local';
    const subject = `Your invite to ${guildName} was revoked`;
    const text = `Your invite to ${guildName} has been revoked.`;
    if (this.transporter) {
      await this.transporter.sendMail({ from, to, subject, text });
      this.logger.log(`Revoke email sent to ${to}`);
    } else {
      this.logger.log(`Revoke (not sent) -> to: ${to}, subject: ${subject}`);
    }
  }
}
