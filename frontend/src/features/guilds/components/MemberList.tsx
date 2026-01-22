'use client'

import { useState } from 'react'
import Image from 'next/image'
import { MoreVertical, UserMinus, Shield, User } from 'lucide-react'
import { RoleBadge } from './RoleBadge'
import { Button } from '@/components/ui/Button'
import type { GuildMember, GuildRole } from '../types'

interface MemberListProps {
  members: GuildMember[]
  currentUserRole?: GuildRole
  onChangeRole?: (memberId: string, newRole: GuildRole) => void
  onRemoveMember?: (memberId: string) => void
}

export function MemberList({
  members,
  currentUserRole = 'member',
  onChangeRole,
  onRemoveMember
}: MemberListProps) {
  const [activeMenu, setActiveMenu] = useState<string | null>(null)

  const canManageMembers = currentUserRole === 'owner' || currentUserRole === 'admin'

  const handleRoleChange = (memberId: string, newRole: GuildRole) => {
    onChangeRole?.(memberId, newRole)
    setActiveMenu(null)
  }

  const handleRemove = (memberId: string) => {
    onRemoveMember?.(memberId)
    setActiveMenu(null)
  }

  return (
    <div className="space-y-3">
      {members.map((member) => (
        <div
          key={member.id}
          className="flex items-center justify-between p-4 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 hover:border-indigo-500 dark:hover:border-indigo-400 transition-colors"
        >
          <div className="flex items-center gap-3 flex-1">
            {/* Avatar */}
            <div className="w-10 h-10 rounded-full bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center text-white font-semibold">
              {member.avatar ? (
                <Image src={member.avatar} alt={member.username} width={40} height={40} className="w-full h-full object-cover rounded-full" />
              ) : (
                member.username.charAt(0).toUpperCase()
              )}
            </div>

            {/* Member Info */}
            <div className="flex-1">
              <h4 className="font-semibold text-gray-900 dark:text-white">
                {member.username}
              </h4>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                Joined {new Date(member.joinedAt).toLocaleDateString()}
              </p>
            </div>

            {/* Reputation */}
            <div className="hidden sm:flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
              <span className="font-medium">{member.reputation}</span>
              <span className="text-xs">REP</span>
            </div>

            {/* Role Badge */}
            <RoleBadge role={member.role} />
          </div>

          {/* Actions Menu */}
          {canManageMembers && member.role !== 'owner' && (
            <div className="relative ml-2">
              <button
                onClick={() => setActiveMenu(activeMenu === member.id ? null : member.id)}
                className="p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
              >
                <MoreVertical className="w-4 h-4" />
              </button>

              {activeMenu === member.id && (
                <>
                  <div
                    className="fixed inset-0 z-10"
                    onClick={() => setActiveMenu(null)}
                  />
                  <div className="absolute right-0 mt-2 w-48 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 py-1 z-20">
                    {currentUserRole === 'owner' && (
                      <>
                        {member.role !== 'admin' && (
                          <button
                            onClick={() => handleRoleChange(member.id, 'admin')}
                            className="w-full px-4 py-2 text-left text-sm hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center gap-2"
                          >
                            <Shield className="w-4 h-4" />
                            Promote to Admin
                          </button>
                        )}
                        {member.role === 'admin' && (
                          <button
                            onClick={() => handleRoleChange(member.id, 'member')}
                            className="w-full px-4 py-2 text-left text-sm hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center gap-2"
                          >
                            <User className="w-4 h-4" />
                            Demote to Member
                          </button>
                        )}
                      </>
                    )}
                    <button
                      onClick={() => handleRemove(member.id)}
                      className="w-full px-4 py-2 text-left text-sm text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 flex items-center gap-2"
                    >
                      <UserMinus className="w-4 h-4" />
                      Remove Member
                    </button>
                  </div>
                </>
              )}
            </div>
          )}
        </div>
      ))}
    </div>
  )
}
