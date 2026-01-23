'use client'

import { useState } from 'react'
import { useRouter } from 'next/navigation'
import { ArrowLeft, CheckCircle } from 'lucide-react'
import Link from 'next/link'
import { useGuildStore } from '@/store/guildStore'
import { GuildForm } from '@/features/guilds/components/GuildForm'
import { Button } from '@/components/ui/Button'
import type { CreateGuildFormData } from '@/features/guilds/types'

export default function CreateGuildPage() {
  const router = useRouter()
  const { createGuild } = useGuildStore()
  const [isLoading, setIsLoading] = useState(false)
  const [showSuccess, setShowSuccess] = useState(false)

  const handleSubmit = async (data: CreateGuildFormData) => {
    setIsLoading(true)

    try {
      const guildId = await createGuild(data)
      setShowSuccess(true)

      // Redirect to guild page after 2 seconds
      setTimeout(() => {
        router.push(`/guilds/${guildId}`)
      }, 2000)
    } catch (error) {
      console.error('Failed to create guild:', error)
    } finally {
      setIsLoading(false)
    }
  }

  if (showSuccess) {
    return (
      <div className="min-h-screen bg-gray-50 dark:bg-gray-900 flex items-center justify-center px-4">
        <div className="text-center">
          <CheckCircle className="w-16 h-16 text-green-500 mx-auto mb-4" />
          <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">
            Guild Created Successfully!
          </h2>
          <p className="text-gray-600 dark:text-gray-400">
            Redirecting to your new guild...
          </p>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      <div className="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Back Button */}
        <Link href="/guilds">
          <Button variant="ghost" leftIcon={<ArrowLeft className="w-4 h-4" />} className="mb-6">
            Back to Guilds
          </Button>
        </Link>

        {/* Header */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-2">
            Create a New Guild
          </h1>
          <p className="text-gray-600 dark:text-gray-400">
            Start your community and bring together like-minded members
          </p>
        </div>

        {/* Form Card */}
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
          <GuildForm onSubmit={handleSubmit} isLoading={isLoading} submitLabel="Create Guild" />
        </div>

        {/* Info Section */}
        <div className="mt-6 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-6">
          <h3 className="text-lg font-semibold text-blue-900 dark:text-blue-100 mb-2">
            What happens next?
          </h3>
          <ul className="space-y-2 text-sm text-blue-800 dark:text-blue-200">
            <li className="flex items-start gap-2">
              <span className="text-blue-500 mt-1">•</span>
              <span>You&apos;ll be the owner of the guild with full administrative privileges</span>
            </li>
            <li className="flex items-start gap-2">
              <span className="text-blue-500 mt-1">•</span>
              <span>You can invite members and assign roles (Admin, Member)</span>
            </li>
            <li className="flex items-start gap-2">
              <span className="text-blue-500 mt-1">•</span>
              <span>Create bounties and manage guild activities</span>
            </li>
            <li className="flex items-start gap-2">
              <span className="text-blue-500 mt-1">•</span>
              <span>Build your guild&apos;s reputation through active participation</span>
            </li>
          </ul>
        </div>
      </div>
    </div>
  )
}
