'use client'

import type { GuildRole } from '../types'

interface RoleBadgeProps {
  role: GuildRole
}

const roleStyles: Record<GuildRole, string> = {
  owner: 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200',
  admin: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
  member: 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200'
}

const roleLabels: Record<GuildRole, string> = {
  owner: 'Owner',
  admin: 'Admin',
  member: 'Member'
}

export function RoleBadge({ role }: RoleBadgeProps) {
  return (
    <span className={`px-2 py-1 rounded-full text-xs font-medium ${roleStyles[role]}`}>
      {roleLabels[role]}
    </span>
  )
}
