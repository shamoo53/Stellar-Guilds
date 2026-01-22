'use client'

import Link from 'next/link'
import Image from 'next/image'
import { Users, TrendingUp, Calendar } from 'lucide-react'
import { Card } from '@/components/ui/Card'
import type { Guild } from '@/types/ui'

interface GuildCardProps {
  guild: Guild
}

const tierColors = {
  bronze: 'bg-amber-700 text-amber-100',
  silver: 'bg-gray-400 text-gray-900',
  gold: 'bg-yellow-500 text-yellow-900',
  platinum: 'bg-purple-500 text-purple-100'
}

export function GuildCard({ guild }: GuildCardProps) {
  return (
    <Link href={`/guilds/${guild.id}`}>
      <Card className="hover:shadow-lg transition-shadow cursor-pointer h-full">
        <div className="flex flex-col h-full">
          {/* Logo and Tier Badge */}
          <div className="flex items-start justify-between mb-4">
            <div className="w-16 h-16 rounded-lg bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center text-white text-2xl font-bold">
              {guild.logo ? (
                <Image src={guild.logo} alt={guild.name} width={64} height={64} className="w-full h-full object-cover rounded-lg" />
              ) : (
                guild.name.charAt(0)
              )}
            </div>
            <span className={`px-3 py-1 rounded-full text-xs font-semibold uppercase ${tierColors[guild.tier]}`}>
              {guild.tier}
            </span>
          </div>

          {/* Guild Info */}
          <div className="flex-1">
            <h3 className="text-xl font-bold text-gray-900 dark:text-white mb-2">
              {guild.name}
            </h3>
            <p className="text-gray-600 dark:text-gray-400 text-sm line-clamp-2 mb-4">
              {guild.description}
            </p>
          </div>

          {/* Stats */}
          <div className="flex items-center justify-between text-sm text-gray-500 dark:text-gray-400 pt-4 border-t border-gray-200 dark:border-gray-700">
            <div className="flex items-center gap-1">
              <Users className="w-4 h-4" />
              <span>{guild.memberCount}</span>
            </div>
            <div className="flex items-center gap-1">
              <TrendingUp className="w-4 h-4" />
              <span>{guild.reputation}</span>
            </div>
            <div className="flex items-center gap-1">
              <Calendar className="w-4 h-4" />
              <span>{new Date(guild.createdAt).getFullYear()}</span>
            </div>
          </div>
        </div>
      </Card>
    </Link>
  )
}
