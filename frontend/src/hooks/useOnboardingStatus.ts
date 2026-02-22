import { useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { useOnboardingStore } from '@/store/onboardingStore';

/**
 * Custom hook to check onboarding status and redirect if needed
 * @param redirectTo - Where to redirect if onboarding is not complete (default: '/onboarding')
 */
export const useOnboardingStatus = (redirectTo: string = '/onboarding') => {
  const router = useRouter();
  const { isOnboardingComplete } = useOnboardingStore();

  // Check onboarding status and redirect if needed
  useEffect(() => {
    if (!isOnboardingComplete) {
      router.push(redirectTo);
    }
  }, [isOnboardingComplete, redirectTo, router]);

  return { isOnboardingComplete };
};

/**
 * Hook to determine if onboarding should be shown
 * @returns boolean indicating if onboarding is incomplete
 */
export const useShouldShowOnboarding = () => {
  const { isOnboardingComplete } = useOnboardingStore();
  return !isOnboardingComplete;
};