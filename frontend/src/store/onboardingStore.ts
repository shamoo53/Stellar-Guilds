import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export type OnboardingStep =
  | 'welcome'
  | 'wallet'
  | 'profile'
  | 'guilds'
  | 'bounties'
  | 'completed';

interface OnboardingState {
  currentStep: OnboardingStep;
  completedSteps: OnboardingStep[];
  isOnboardingComplete: boolean;
  totalSteps: number;
  
  // Actions
  goToStep: (step: OnboardingStep) => void;
  nextStep: () => void;
  prevStep: () => void;
  markStepComplete: (step: OnboardingStep) => void;
  resetOnboarding: () => void;
  initializeOnboarding: () => void;
}

// Define the sequence of onboarding steps
const ONBOARDING_STEPS: OnboardingStep[] = [
  'welcome',
  'wallet',
  'profile',
  'guilds',
  'bounties',
  'completed'
];

export const useOnboardingStore = create<OnboardingState>()(
  persist(
    (set, get) => ({
      currentStep: 'welcome',
      completedSteps: [],
      isOnboardingComplete: false,
      totalSteps: ONBOARDING_STEPS.length - 1, // Exclude 'completed' from count
      
      goToStep: (step) => {
        set({ currentStep: step });
      },
      
      nextStep: () => {
        const { currentStep, completedSteps } = get();
        const currentIndex = ONBOARDING_STEPS.indexOf(currentStep);
        
        if (currentIndex < ONBOARDING_STEPS.length - 1) {
          const nextStep = ONBOARDING_STEPS[currentIndex + 1];
          
          // Mark current step as complete if not already marked
          if (!completedSteps.includes(currentStep) && currentStep !== 'completed') {
            set({
              completedSteps: [...completedSteps, currentStep],
              currentStep: nextStep
            });
          } else {
            set({ currentStep: nextStep });
          }
        }
      },
      
      prevStep: () => {
        const { currentStep } = get();
        const currentIndex = ONBOARDING_STEPS.indexOf(currentStep);
        
        if (currentIndex > 0) {
          const prevStep = ONBOARDING_STEPS[currentIndex - 1];
          set({ currentStep: prevStep });
        }
      },
      
      markStepComplete: (step) => {
        const { completedSteps } = get();
        if (!completedSteps.includes(step)) {
          set({
            completedSteps: [...completedSteps, step],
            isOnboardingComplete: step === 'completed'
          });
        }
      },
      
      resetOnboarding: () => {
        set({
          currentStep: 'welcome',
          completedSteps: [],
          isOnboardingComplete: false
        });
      },
      
      initializeOnboarding: () => {
        const { completedSteps, currentStep } = get();
        // If no steps are completed, we're at the beginning
        if (completedSteps.length === 0 && currentStep === 'welcome') {
          // No action needed, already initialized
        }
      }
    }),
    {
      name: 'onboarding-storage', // Unique name for localStorage
      partialize: (state) => ({ 
        completedSteps: state.completedSteps,
        isOnboardingComplete: state.isOnboardingComplete
      }) // Only persist completion status, not current step
    }
  )
);

// Helper function to calculate progress percentage
export const getOnboardingProgress = (): number => {
  const { completedSteps, totalSteps } = useOnboardingStore.getState();
  return Math.min(100, Math.floor((completedSteps.length / totalSteps) * 100));
};