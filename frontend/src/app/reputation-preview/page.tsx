import React from 'react';
import { ReputationCard } from '@/features/profile/components/ReputationCard';
import { UserProfile, ReputationTier } from '@/features/profile/types';
import { mockUser } from '@/features/profile/mockData';

const tiers: ReputationTier[] = ['Bronze', 'Silver', 'Gold', 'Platinum', 'Diamond'];

export default function ReputationPreviewPage() {
  return (
    <div className="min-h-screen bg-slate-50 py-8 px-4 sm:px-6 lg:px-8">
      <div className="max-w-4xl mx-auto space-y-8">
        <div className="text-center space-y-2">
          <h1 className="text-3xl font-bold text-slate-900">Reputation Tier Styles</h1>
          <p className="text-slate-600">Preview of all reputation card tier variations</p>
        </div>

        <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
          {tiers.map((tier) => {
            const tierUser: UserProfile = {
              ...mockUser,
              tier: tier,
              reputationScore: getScoreForTier(tier),
              nextTierScore: getNextScoreForTier(tier),
            };

            return (
              <div key={tier} className="space-y-2">
                <h2 className="text-lg font-semibold text-slate-700">{tier}</h2>
                <ReputationCard user={tierUser} />
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}

function getScoreForTier(tier: ReputationTier): number {
  switch (tier) {
    case 'Bronze': return 100;
    case 'Silver': return 400;
    case 'Gold': return 750;
    case 'Platinum': return 1500;
    case 'Diamond': return 3000;
    default: return 0;
  }
}

function getNextScoreForTier(tier: ReputationTier): number {
  switch (tier) {
    case 'Bronze': return 300;
    case 'Silver': return 600;
    case 'Gold': return 1000;
    case 'Platinum': return 2500;
    case 'Diamond': return 5000;
    default: return 100;
  }
}
