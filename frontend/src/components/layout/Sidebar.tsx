'use client'

import React from 'react'
import { Users, Trophy, Vote, User } from 'lucide-react'
import { useSidebarStore } from '@/store/sidebarStore'
import { cn } from '@/lib/utils'
import { mockGuilds, mockBounties } from '@/lib/mocks'

interface SidebarItem {
  id: string
  label: string
  href: string
  icon: React.ReactNode
  isActive?: boolean
}

interface SidebarProps {
  className?: string
}

const Sidebar = ({ className }: SidebarProps) => {
  const { isOpen, closeSidebar } = useSidebarStore()
  
  const navItems: SidebarItem[] = [
    { id: 'guilds', label: 'Guilds', href: '#', icon: <Users size={20} />, isActive: true },
    { id: 'bounties', label: 'Bounties', href: '#', icon: <Trophy size={20} /> },
    { id: 'governance', label: 'Governance', href: '#', icon: <Vote size={20} /> },
    { id: 'profile', label: 'Profile', href: '/profile', icon: <User size={20} /> },
    // social features
    { id: 'feed', label: 'Feed', href: '/social/feed', icon: <Users size={20} /> },
    { id: 'messages', label: 'Messages', href: '/social/messages', icon: <Users size={20} /> },
    { id: 'forum', label: 'Forum', href: '/social/forum', icon: <Users size={20} /> },
  ]

  const NavItem = ({ item }: { item: SidebarItem }) => (
    <a
      href={item.href}
      className={cn(
        "flex items-center space-x-3 px-3 py-2 rounded-lg transition-colors",
        item.isActive 
          ? "bg-stellar-lightNavy text-gold-400 border-l-4 border-gold-500" 
          : "text-stellar-slate hover:bg-stellar-lightNavy hover:text-stellar-lightSlate"
      )}
      onClick={() => closeSidebar()}
    >
      {item.icon}
      <span className="font-medium">{item.label}</span>
    </a>
  )

  return (
    <>
      <aside 
        className={cn(
          "fixed inset-y-0 left-0 z-40 w-64 bg-stellar-darkNavy border-r border-stellar-lightNavy transform transition-transform duration-300 ease-in-out md:translate-x-0 md:static md:z-auto",
          isOpen ? "translate-x-0" : "-translate-x-full",
          className
        )}
      >
        <div className="flex flex-col h-full pt-16">
          <nav className="flex-1 px-4 py-6 space-y-2">
            {navItems.map((item) => (
              <NavItem key={item.id} item={item} />
            ))}
          </nav>
          
          <div className="p-4 border-t border-stellar-lightNavy">
            <div className="text-xs text-stellar-slate uppercase tracking-wider mb-2">
              Quick Stats
            </div>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-stellar-slate">Active Guilds</span>
                <span className="text-stellar-white font-medium">{mockGuilds.length}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-stellar-slate">Open Bounties</span>
                <span className="text-stellar-white font-medium">
                  {mockBounties.filter(b => b.status === 'open').length}
                </span>
              </div>
            </div>
          </div>
        </div>
      </aside>

      {/* Overlay for mobile */}
      {isOpen && (
        <div 
          className="fixed inset-0 z-30 bg-black/50 md:hidden"
          onClick={() => closeSidebar()}
        />
      )}
    </>
  )
}

export { Sidebar }