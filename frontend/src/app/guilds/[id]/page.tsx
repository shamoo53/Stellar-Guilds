'use client'

import { useEffect, useState } from 'react'
import { useParams, useRouter } from 'next/navigation'
import Link from 'next/link'
import Image from 'next/image'
import { ArrowLeft, Settings, UserPlus, Activity, Users as UsersIcon } from 'lucide-react'
import { useGuildStore } from '@/store/guildStore'
import { GuildStats } from '@/features/guilds/components/GuildStats'
import { MemberList } from '@/features/guilds/components/MemberList'
import { Button } from '@/components/ui/Button'
import type { GuildRole } from '@/features/guilds/types'

type TabType = 'overview' | 'members' | 'activity'

export default function GuildDetailPage() {
  const params = useParams()
  const router = useRouter()
  const guildId = params.id as string
  const { currentGuild, fetchGuildById, changeMemberRole, removeMember, isLoading } = useGuildStore()
  const [activeTab, setActiveTab] = useState<TabType>('overview')
  const [currentUserRole] = useState<GuildRole>('owner') // Mock: current user is owner

  useEffect(() => {
    if (guildId) {
      fetchGuildById(guildId)
    }
  }, [guildId, fetchGuildById])

  if (isLoading) {
    return (
      <div className="min-h-screen bg-gray-50 dark:bg-gray-900 flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600 mx-auto mb-4"></div>
          <p className="text-gray-600 dark:text-gray-400">Loading guild...</p>
        </div>
      </div>
    )
  }

  if (!currentGuild) {
    return (
      <div className="min-h-screen bg-gray-50 dark:bg-gray-900 flex items-center justify-center">
        <div className="text-center">
          <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">
            Guild Not Found
          </h2>
          <p className="text-gray-600 dark:text-gray-400 mb-4">
            The guild you&apos;re looking for doesn&apos;t exist
          </p>
          <Link href="/guilds">
            <Button variant="primary">Back to Guilds</Button>
          </Link>
        </div>
      </div>
    )
  }

  const handleRoleChange = (memberId: string, newRole: GuildRole) => {
    changeMemberRole(guildId, memberId, newRole)
  }

  const handleRemoveMember = (memberId: string) => {
    removeMember(guildId, memberId)
  }

  const tabs: { id: TabType; label: string; icon: React.ReactNode }[] = [
    { id: 'overview', label: 'Overview', icon: <Activity className="w-4 h-4" /> },
    { id: 'members', label: 'Members', icon: <UsersIcon className="w-4 h-4" /> },
    { id: 'activity', label: 'Activity', icon: <Activity className="w-4 h-4" /> }
  ]

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Back Button */}
        <Link href="/guilds">
          <Button variant="ghost" leftIcon={<ArrowLeft className="w-4 h-4" />} className="mb-6">
            Back to Guilds
          </Button>
        </Link>

        {/* Guild Header */}
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6 mb-6">
          <div className="flex flex-col md:flex-row md:items-start md:justify-between gap-6">
            <div className="flex items-start gap-4">
              {/* Logo */}
              <div className="w-20 h-20 rounded-lg bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center text-white text-3xl font-bold flex-shrink-0">
                {currentGuild.logo ? (
                  <Image
                    src={currentGuild.logo}
                    alt={currentGuild.name}
                    width={80}
                    height={80}
                    className="w-full h-full object-cover rounded-lg"
                  />
                ) : (
                  currentGuild.name.charAt(0)
                )}
              </div>

              {/* Info */}
              <div className="flex-1">
                <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-2">
                  {currentGuild.name}
                </h1>
                <p className="text-gray-600 dark:text-gray-400">
                  {currentGuild.description}
                </p>
                {currentGuild.category && (
                  <span className="inline-block mt-2 px-3 py-1 bg-indigo-100 dark:bg-indigo-900 text-indigo-800 dark:text-indigo-200 rounded-full text-sm font-medium">
                    {currentGuild.category}
                  </span>
                )}
              </div>
            </div>

            {/* Actions */}
            <div className="flex gap-3">
              {(currentUserRole === 'owner' || currentUserRole === 'admin') && (
                <Link href={`/guilds/${guildId}/settings`}>
                  <Button variant="outline" leftIcon={<Settings className="w-4 h-4" />}>
                    Settings
                  </Button>
                </Link>
              )}
            </div>
          </div>
        </div>

        {/* Stats */}
        <div className="mb-6">
          <GuildStats
            memberCount={currentGuild.memberCount}
            reputation={currentGuild.reputation}
            createdAt={currentGuild.createdAt}
            tier={currentGuild.tier}
          />
        </div>

        {/* Tabs */}
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 mb-6">
          <div className="border-b border-gray-200 dark:border-gray-700">
            <nav className="flex gap-4 px-6">
              {tabs.map((tab) => (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id)}
                  className={`flex items-center gap-2 py-4 px-2 border-b-2 font-medium text-sm transition-colors ${
                    activeTab === tab.id
                      ? 'border-indigo-500 text-indigo-600 dark:text-indigo-400'
                      : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300'
                  }`}
                >
                  {tab.icon}
                  {tab.label}
                </button>
              ))}
            </nav>
          </div>

          <div className="p-6">
            {/* Overview Tab */}
            {activeTab === 'overview' && (
              <div>
                <h2 className="text-xl font-bold text-gray-900 dark:text-white mb-4">
                  About This Guild
                </h2>
                <p className="text-gray-600 dark:text-gray-400 mb-6">
                  {currentGuild.description}
                </p>

                <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-3">
                  Recent Activity
                </h3>
                <div className="space-y-3">
                  {currentGuild.activities.slice(0, 5).map((activity) => (
                    <div
                      key={activity.id}
                      className="flex items-start gap-3 p-3 bg-gray-50 dark:bg-gray-700 rounded-lg"
                    >
                      <Activity className="w-5 h-5 text-gray-400 mt-0.5" />
                      <div className="flex-1">
                        <p className="text-sm text-gray-900 dark:text-white">
                          {activity.description}
                        </p>
                        <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                          {new Date(activity.timestamp).toLocaleString()}
                        </p>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Members Tab */}
            {activeTab === 'members' && (
              <div>
                <div className="flex items-center justify-between mb-4">
                  <h2 className="text-xl font-bold text-gray-900 dark:text-white">
                    Members ({currentGuild.members.length})
                  </h2>
                  {(currentUserRole === 'owner' || currentUserRole === 'admin') && (
                    <Button variant="outline" leftIcon={<UserPlus className="w-4 h-4" />}>
                      Invite Members
                    </Button>
                  )}
                </div>
                <MemberList
                  members={currentGuild.members}
                  currentUserRole={currentUserRole}
                  onChangeRole={handleRoleChange}
                  onRemoveMember={handleRemoveMember}
                />
              </div>
            )}

            {/* Activity Tab */}
            {activeTab === 'activity' && (
              <div>
                <h2 className="text-xl font-bold text-gray-900 dark:text-white mb-4">
                  Activity Timeline
                </h2>
                <div className="space-y-3">
                  {currentGuild.activities.map((activity) => (
                    <div
                      key={activity.id}
                      className="flex items-start gap-3 p-4 bg-white dark:bg-gray-700 rounded-lg border border-gray-200 dark:border-gray-600"
                    >
                      <Activity className="w-5 h-5 text-gray-400 mt-0.5" />
                      <div className="flex-1">
                        <p className="text-sm text-gray-900 dark:text-white font-medium">
                          {activity.description}
                        </p>
                        {activity.username && (
                          <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                            by {activity.username}
                          </p>
                        )}
                        <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                          {new Date(activity.timestamp).toLocaleString()}
                        </p>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}
