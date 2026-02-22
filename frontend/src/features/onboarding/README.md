# Onboarding Feature

The onboarding feature provides a guided experience for new users to learn about the Stellar Guilds platform and set up their accounts.

## Components

### Store
- `onboardingStore.ts`: Manages the onboarding state, including current step, completed steps, and progress

### Pages
- `OnboardingPage.tsx`: Main container that manages the flow between different onboarding steps

### Step Components
- `WelcomeStep.tsx`: Introduction to the platform
- `WalletSetupStep.tsx`: Guided wallet connection process
- `ProfileSetupStep.tsx`: Profile creation form
- `GuildsIntroductionStep.tsx`: Information about guilds and communities
- `BountiesIntroductionStep.tsx`: Information about bounties and rewards
- `CompletionStep.tsx`: Final completion screen

### Utility Components
- `OnboardingProgressIndicator.tsx`: Visual progress indicator showing current step
- `OnboardingTrigger.tsx`: Floating button to start or resume onboarding

### Hooks
- `useOnboardingStatus.ts`: Hook to check onboarding status and redirect if needed

## Usage

To integrate onboarding into the app:

1. Add the `/onboarding` route to your navigation
2. Use the `OnboardingTrigger` component to provide easy access to onboarding
3. Use the `useOnboardingStatus` hook to conditionally redirect users

## State Management

The onboarding state is persisted using zustand's persistence middleware, ensuring that users can resume where they left off if they navigate away or refresh the page.

## Customization

The onboarding flow can be customized by:
- Modifying the `ONBOARDING_STEPS` array in the store
- Adding new step components
- Adjusting the content and styling of existing components