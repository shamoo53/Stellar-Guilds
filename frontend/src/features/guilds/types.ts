export type GuildRole = 'owner' | 'admin' | 'member'

export interface GuildMember {
  id: string
  username: string
  avatar?: string
  role: GuildRole
  joinedAt: string
  reputation: number
}

export interface GuildActivity {
  id: string
  type: 'member_joined' | 'member_left' | 'role_changed' | 'bounty_created' | 'bounty_completed'
  description: string
  userId?: string
  username?: string
  timestamp: string
}

export interface GuildDetail {
  id: string
  name: string
  description: string
  tier: 'bronze' | 'silver' | 'gold' | 'platinum'
  memberCount: number
  reputation: number
  createdAt: string
  logo?: string
  members: GuildMember[]
  activities: GuildActivity[]
  category?: string
}

export interface CreateGuildFormData {
  name: string
  description: string
  logo?: string
  category?: string
  initialMembers?: string[]
}

export interface GuildFilters {
  search: string
  category?: string
  tier?: string
}
