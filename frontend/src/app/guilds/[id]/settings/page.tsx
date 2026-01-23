'use client'

import { useEffect, useState } from 'react'
import { useParams, useRouter } from 'next/navigation'
import Link from 'next/link'
import { ArrowLeft, Save, Trash2 } from 'lucide-react'
import { useGuildStore } from '@/store/guildStore'
import { GuildForm } from '@/features/guilds/components/GuildForm'
import { Button } from '@/components/ui/Button'
import { Modal } from '@/components/ui/Modal'
import type { CreateGuildFormData } from '@/features/guilds/types'

export default function GuildSettingsPage() {
  const params = useParams()
  const router = useRouter()
  const guildId = params.id as string
  const { currentGuild, fetchGuildById, updateGuild, isLoading } = useGuildStore()
  const [isSaving, setIsSaving] = useState(false)
  const [showDeleteModal, setShowDeleteModal] = useState(false)
  const [showSuccessMessage, setShowSuccessMessage] = useState(false)

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
          <p className="text-gray-600 dark:text-gray-400">Loading guild settings...</p>
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

  const handleSubmit = async (data: CreateGuildFormData) => {
    setIsSaving(true)

    try {
      updateGuild(guildId, data)
      setShowSuccessMessage(true)

      // Hide success message after 3 seconds
      setTimeout(() => {
        setShowSuccessMessage(false)
      }, 3000)
    } catch (error) {
      console.error('Failed to update guild:', error)
    } finally {
      setIsSaving(false)
    }
  }

  const handleDeleteGuild = () => {
    // Simulate delete - in production this would call an API
    console.log('Deleting guild:', guildId)
    setShowDeleteModal(false)
    router.push('/guilds')
  }

  const initialData: Partial<CreateGuildFormData> = {
    name: currentGuild.name,
    description: currentGuild.description,
    logo: currentGuild.logo || '',
    category: currentGuild.category
  }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      <div className="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Back Button */}
        <Link href={`/guilds/${guildId}`}>
          <Button variant="ghost" leftIcon={<ArrowLeft className="w-4 h-4" />} className="mb-6">
            Back to Guild
          </Button>
        </Link>

        {/* Header */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-2">
            Guild Settings
          </h1>
          <p className="text-gray-600 dark:text-gray-400">
            Manage your guild&apos;s information and preferences
          </p>
        </div>

        {/* Success Message */}
        {showSuccessMessage && (
          <div className="mb-6 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg p-4">
            <p className="text-green-800 dark:text-green-200 font-medium">
              Guild settings updated successfully!
            </p>
          </div>
        )}

        {/* Settings Form */}
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6 mb-6">
          <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-6">
            Basic Information
          </h2>
          <GuildForm
            onSubmit={handleSubmit}
            initialData={initialData}
            isLoading={isSaving}
            submitLabel="Save Changes"
          />
        </div>

        {/* Danger Zone */}
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-red-200 dark:border-red-800 p-6">
          <h2 className="text-xl font-semibold text-red-600 dark:text-red-400 mb-4">
            Danger Zone
          </h2>
          <div className="flex items-start justify-between gap-4">
            <div>
              <h3 className="font-medium text-gray-900 dark:text-white mb-1">
                Delete Guild
              </h3>
              <p className="text-sm text-gray-600 dark:text-gray-400">
                Once you delete a guild, there is no going back. This action cannot be undone.
              </p>
            </div>
            <Button
              variant="danger"
              leftIcon={<Trash2 className="w-4 h-4" />}
              onClick={() => setShowDeleteModal(true)}
            >
              Delete Guild
            </Button>
          </div>
        </div>

        {/* Delete Confirmation Modal */}
        <Modal
          isOpen={showDeleteModal}
          onClose={() => setShowDeleteModal(false)}
          title="Delete Guild"
        >
          <div className="space-y-4">
            <p className="text-gray-600 dark:text-gray-400">
              Are you sure you want to delete <strong>{currentGuild.name}</strong>? This action
              cannot be undone and all guild data will be permanently removed.
            </p>
            <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
              <h4 className="font-semibold text-red-900 dark:text-red-100 mb-2">
                This will permanently delete:
              </h4>
              <ul className="text-sm text-red-800 dark:text-red-200 space-y-1">
                <li>• All guild members and roles</li>
                <li>• Guild activity history</li>
                <li>• All associated bounties</li>
                <li>• Guild reputation and stats</li>
              </ul>
            </div>
            <div className="flex gap-3 pt-4">
              <Button
                variant="outline"
                onClick={() => setShowDeleteModal(false)}
                className="flex-1"
              >
                Cancel
              </Button>
              <Button
                variant="danger"
                onClick={handleDeleteGuild}
                className="flex-1"
              >
                Delete Permanently
              </Button>
            </div>
          </div>
        </Modal>
      </div>
    </div>
  )
}
