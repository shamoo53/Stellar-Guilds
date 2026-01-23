'use client'

import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import { Input } from '@/components/ui/Input'
import { Button } from '@/components/ui/Button'
import type { CreateGuildFormData } from '../types'

const guildSchema = z.object({
  name: z.string().min(3, 'Guild name must be at least 3 characters').max(50, 'Guild name must be less than 50 characters'),
  description: z.string().min(10, 'Description must be at least 10 characters').max(500, 'Description must be less than 500 characters'),
  logo: z.string().url('Must be a valid URL').optional().or(z.literal('')),
  category: z.string().min(1, 'Please select a category')
})

interface GuildFormProps {
  onSubmit: (data: CreateGuildFormData) => void
  initialData?: Partial<CreateGuildFormData>
  isLoading?: boolean
  submitLabel?: string
}

const categories = [
  'Development',
  'DeFi',
  'Education',
  'Gaming',
  'NFT',
  'DAO',
  'Social',
  'Other'
]

export function GuildForm({
  onSubmit,
  initialData,
  isLoading = false,
  submitLabel = 'Create Guild'
}: GuildFormProps) {
  const {
    register,
    handleSubmit,
    formState: { errors }
  } = useForm<CreateGuildFormData>({
    resolver: zodResolver(guildSchema),
    defaultValues: initialData
  })

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="space-y-6">
      <Input
        label="Guild Name"
        placeholder="Enter guild name"
        error={errors.name?.message}
        {...register('name')}
      />

      <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Description
        </label>
        <textarea
          placeholder="Describe your guild's purpose and goals"
          rows={4}
          className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500 dark:bg-gray-800 dark:text-white"
          {...register('description')}
        />
        {errors.description && (
          <p className="mt-1 text-sm text-red-600">{errors.description.message}</p>
        )}
      </div>

      <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Category
        </label>
        <select
          className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500 dark:bg-gray-800 dark:text-white"
          {...register('category')}
        >
          <option value="">Select a category</option>
          {categories.map((cat) => (
            <option key={cat} value={cat}>
              {cat}
            </option>
          ))}
        </select>
        {errors.category && (
          <p className="mt-1 text-sm text-red-600">{errors.category.message}</p>
        )}
      </div>

      <Input
        label="Logo URL (Optional)"
        placeholder="https://example.com/logo.png"
        error={errors.logo?.message}
        helperText="Provide a URL to your guild's logo image"
        {...register('logo')}
      />

      <div className="flex gap-3 pt-4">
        <Button
          type="submit"
          variant="primary"
          size="lg"
          isLoading={isLoading}
          className="flex-1"
        >
          {submitLabel}
        </Button>
      </div>
    </form>
  )
}
