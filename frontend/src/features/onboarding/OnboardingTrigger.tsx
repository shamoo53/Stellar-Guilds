'use client';

import React, { useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { useOnboardingStore } from '@/store/onboardingStore';
import { Button } from '@/components/ui/Button';

/**
 * Component that can be placed anywhere in the app to trigger onboarding
 * if the user hasn't completed it yet
 */
const OnboardingTrigger = () => {
  const router = useRouter();
  const { isOnboardingComplete, completedSteps } = useOnboardingStore();

  // Optionally redirect to onboarding if not complete
  useEffect(() => {
    if (!isOnboardingComplete && completedSteps.length === 0) {
      // Only redirect if this is the first time and onboarding hasn't started
      // You can customize this logic based on your needs
    }
  }, [isOnboardingComplete, completedSteps]);

  const startOnboarding = () => {
    router.push('/onboarding');
  };

  const resumeOnboarding = () => {
    router.push('/onboarding');
  };

  if (isOnboardingComplete) {
    return null; // Don't show anything if onboarding is complete
  }

  return (
    <div className="fixed bottom-6 right-6 z-50">
      <Button
        onClick={completedSteps.length > 0 ? resumeOnboarding : startOnboarding}
        className="bg-gradient-to-r from-gold-500 to-gold-600 hover:from-gold-400 hover:to-gold-500 text-stellar-navy shadow-lg shadow-gold-500/30"
      >
        {completedSteps.length > 0 
          ? `Resume Onboarding (${completedSteps.length}/5)` 
          : 'Start Onboarding'}
      </Button>
    </div>
  );
};

export default OnboardingTrigger;