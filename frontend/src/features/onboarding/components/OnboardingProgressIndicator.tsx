'use client';

import React from 'react';
import { motion } from 'framer-motion';
import { CheckCircle, Circle, ArrowRight } from 'lucide-react';
import { useOnboardingStore, getOnboardingProgress, OnboardingStep } from '@/store/onboardingStore';

const STEP_LABELS: Record<OnboardingStep, string> = {
  'welcome': 'Welcome',
  'wallet': 'Wallet Setup',
  'profile': 'Profile',
  'guilds': 'Guilds',
  'bounties': 'Bounties',
  'completed': 'Completed'
};

const OnboardingProgressIndicator = () => {
  const { currentStep, completedSteps } = useOnboardingStore();
  const progress = getOnboardingProgress();
  
  const steps: OnboardingStep[] = ['welcome', 'wallet', 'profile', 'guilds', 'bounties'];
  
  return (
    <div className="w-full max-w-2xl mx-auto p-6">
      {/* Progress bar */}
      <div className="mb-8">
        <div className="flex justify-between items-center mb-2">
          <span className="text-sm font-medium text-stellar-slate">Onboarding Progress</span>
          <span className="text-sm font-medium text-gold-400">{progress}%</span>
        </div>
        <div className="w-full bg-stellar-lightNavy rounded-full h-2.5">
          <motion.div 
            className="bg-gradient-to-r from-gold-500 to-gold-600 h-2.5 rounded-full"
            initial={{ width: 0 }}
            animate={{ width: `${progress}%` }}
            transition={{ duration: 0.5, ease: 'easeInOut' }}
          />
        </div>
      </div>
      
      {/* Step indicators */}
      <div className="flex justify-between relative">
        {/* Connecting line */}
        <div className="absolute top-4 left-0 w-full h-0.5 bg-stellar-lightNavy -z-10" />
        <div 
          className="absolute top-4 left-0 h-0.5 bg-gradient-to-r from-gold-500 to-gold-600 -z-10" 
          style={{ width: `${(completedSteps.length / steps.length) * 100}%` }}
        />
        
        {steps.map((step, index) => {
          const isCurrent = currentStep === step;
          const isCompleted = completedSteps.includes(step);
          
          return (
            <div key={step} className="flex flex-col items-center relative z-10">
              <motion.button
                whileHover={{ scale: 1.1 }}
                whileTap={{ scale: 0.95 }}
                className={`w-8 h-8 rounded-full flex items-center justify-center ${
                  isCurrent 
                    ? 'bg-gradient-to-r from-gold-500 to-gold-600 text-stellar-navy shadow-lg shadow-gold-500/30' 
                    : isCompleted 
                      ? 'bg-emerald-500 text-white' 
                      : 'bg-stellar-lightNavy text-stellar-slate border border-stellar-slate/20'
                }`}
              >
                {isCompleted ? (
                  <CheckCircle size={16} />
                ) : (
                  <span className="text-xs font-bold">{index + 1}</span>
                )}
              </motion.button>
              
              <span 
                className={`mt-2 text-xs text-center max-w-[80px] ${
                  isCurrent ? 'text-gold-400 font-medium' : 'text-stellar-slate'
                }`}
              >
                {STEP_LABELS[step]}
              </span>
            </div>
          );
        })}
      </div>
    </div>
  );
};

export default OnboardingProgressIndicator;