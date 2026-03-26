// UI Components
export { Button } from './ui/Button'
export { Input } from './ui/Input'
export { Card } from './ui/Card'
export { Modal } from './ui/Modal'

// Error Boundary
export { ErrorBoundary } from './ErrorBoundary'

// Bounty Components
export { BountyCard } from './bounty'
export type { BountyStatus } from './bounty'

// Guild Components
export { GuildCard } from './guild'
export type { GuildTier } from './guild'

// Layout Components
export { Header } from './layout/Header'
export { Sidebar } from './layout/Sidebar'
export { Footer } from './layout/Footer'

// Utilities
export { cn } from '@/lib/utils'

// Mock Data
export { mockGuilds, mockBounties, mockProfiles, generateRandomGuild, generateRandomBounty } from '@/lib/mocks'

// Wallet Components
export { WalletConnectButton, WalletModal, TransactionConfirmation, WalletNotInstalled } from './WalletConnector'

// Profile Enhancements
export { ProfileSocialSection } from './Profile/ProfileSocialSection'

// Messaging
export { ChatWindow } from './Messaging/ChatWindow'
export { MessageInput } from './Messaging/MessageInput'

// Forum
export { ThreadList } from './Forum/ThreadList'
export { PostItem } from './Forum/PostItem'
export { ModerationTools } from './Forum/ModerationTools'

// Social utilities
export { NotificationBell } from '@/features/social/components/NotificationBell'
// Hooks
export { useWallet } from '@/hooks/useWallet'
export { useWalletModal } from '@/hooks/useWalletModal'

// Security
export { SecurityDashboard } from './Security/SecurityDashboard'
export { useSecurity } from '@/hooks/useSecurity'

// Stores
export { useSidebarStore } from '@/store/sidebarStore'
export { useThemeStore } from '@/store/themeStore'
export { useWalletStore } from '@/store/walletStore'
export { useSecurityStore } from '@/store/securityStore'