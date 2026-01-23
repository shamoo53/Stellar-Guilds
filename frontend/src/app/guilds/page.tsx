'use client'

import { useState, useEffect } from 'react'
import Link from 'next/link'
import { Search, Plus, Filter } from 'lucide-react'
import { useGuildStore } from '@/store/guildStore'
import { GuildCard } from '@/features/guilds/components/GuildCard'
import { Button } from '@/components/ui/Button'
import { Input } from '@/components/ui/Input'

export default function GuildsPage() {
  const { guilds, fetchGuilds } = useGuildStore()
  const [searchQuery, setSearchQuery] = useState('')
  const [selectedCategory, setSelectedCategory] = useState<string>('')
  const [selectedTier, setSelectedTier] = useState<string>('')

  useEffect(() => {
    fetchGuilds()
  }, [fetchGuilds])

  const categories = ['All', 'Development', 'DeFi', 'Education', 'Gaming', 'NFT', 'DAO', 'Social']
  const tiers = ['All', 'bronze', 'silver', 'gold', 'platinum']

  const filteredGuilds = guilds.filter((guild) => {
    const matchesSearch =
      guild.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      guild.description.toLowerCase().includes(searchQuery.toLowerCase())

    const matchesCategory =
      !selectedCategory || selectedCategory === 'All' || guild.category === selectedCategory

    const matchesTier = !selectedTier || selectedTier === 'All' || guild.tier === selectedTier

    return matchesSearch && matchesCategory && matchesTier
  })

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Header */}
        <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 mb-8">
          <div>
            <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
              Discover Guilds
            </h1>
            <p className="text-gray-600 dark:text-gray-400 mt-1">
              Find and join communities that match your interests
            </p>
          </div>
          <Link href="/guilds/create">
            <Button variant="primary" leftIcon={<Plus className="w-5 h-5" />}>
              Create Guild
            </Button>
          </Link>
        </div>

        {/* Search and Filters */}
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6 mb-6">
          <div className="flex flex-col lg:flex-row gap-4">
            {/* Search */}
            <div className="flex-1">
              <div className="relative">
                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-5 h-5" />
                <input
                  type="text"
                  placeholder="Search guilds..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="w-full pl-10 pr-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500 dark:bg-gray-800 dark:text-white"
                />
              </div>
            </div>

            {/* Category Filter */}
            <div className="w-full lg:w-48">
              <select
                value={selectedCategory}
                onChange={(e) => setSelectedCategory(e.target.value)}
                className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500 dark:bg-gray-800 dark:text-white"
              >
                {categories.map((cat) => (
                  <option key={cat} value={cat === 'All' ? '' : cat}>
                    {cat}
                  </option>
                ))}
              </select>
            </div>

            {/* Tier Filter */}
            <div className="w-full lg:w-48">
              <select
                value={selectedTier}
                onChange={(e) => setSelectedTier(e.target.value)}
                className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500 dark:bg-gray-800 dark:text-white capitalize"
              >
                {tiers.map((tier) => (
                  <option key={tier} value={tier === 'All' ? '' : tier}>
                    {tier}
                  </option>
                ))}
              </select>
            </div>
          </div>
        </div>

        {/* Results Count */}
        <div className="mb-4">
          <p className="text-sm text-gray-600 dark:text-gray-400">
            Showing {filteredGuilds.length} of {guilds.length} guilds
          </p>
        </div>

        {/* Guild Grid */}
        {filteredGuilds.length > 0 ? (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {filteredGuilds.map((guild) => (
              <GuildCard key={guild.id} guild={guild} />
            ))}
          </div>
        ) : (
          <div className="text-center py-12">
            <Filter className="w-12 h-12 text-gray-400 mx-auto mb-4" />
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
              No guilds found
            </h3>
            <p className="text-gray-600 dark:text-gray-400">
              Try adjusting your search or filters
            </p>
          </div>
        )}
      </div>
    </div>
  )
}
