'use client';

import React from 'react';
import { motion } from 'framer-motion';
import { CheckCircle, Star, Gift, Trophy } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { Card } from '@/components/ui/Card';
import { useOnboardingStore } from '@/store/onboardingStore';
import { useRouter } from 'next/navigation';

const CompletionStep = () => {
  const { resetOnboarding, markStepComplete } = useOnboardingStore();
  const router = useRouter();

  const achievements = [
    {
      icon: <CheckCircle className="w-6 h-6 text-emerald-400" />,
      title: "Wallet Connected",
      description: "Ready to interact with the Stellar network"
    },
    {
      icon: <Star className="w-6 h-6 text-yellow-400" />,
      title: "Profile Created",
      description: "Showcase your skills and experience"
    },
    {
      icon: <Gift className="w-6 h-6 text-purple-400" />,
      title: "Guild Ready",
      description: "Join or create communities"
    },
    {
      icon: <Trophy className="w-6 h-6 text-blue-400" />,
      title: "Bounty Prepared",
      description: "Start earning rewards"
    }
  ];

  const handleFinish = () => {
    markStepComplete('completed');
    // Redirect to dashboard or home page
    router.push('/');
  };

  const handleRestart = () => {
    resetOnboarding();
  };

  return (
    <motion.div 
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -20 }}
      className="max-w-3xl mx-auto text-center"
    >
      <motion.div
        initial={{ scale: 0.8, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ delay: 0.2 }}
        className="mb-8"
      >
        <motion.div
          initial={{ scale: 0.5, rotate: -180 }}
          animate={{ scale: 1, rotate: 0 }}
          transition={{ delay: 0.3, type: "spring", stiffness: 200 }}
          className="w-24 h-24 bg-gradient-to-r from-emerald-500 to-emerald-600 rounded-full flex items-center justify-center mx-auto mb-6"
        >
          <CheckCircle className="w-12 h-12 text-white" />
        </motion.div>
        <h1 className="text-4xl md:text-5xl font-bold bg-gradient-to-r from-emerald-400 to-emerald-600 bg-clip-text text-transparent mb-4">
          Congratulations!
        </h1>
        <p className="text-xl text-stellar-slate max-w-2xl mx-auto">
          You've successfully completed the Stellar Guilds onboarding process
        </p>
      </motion.div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-12">
        {achievements.map((achievement, index) => (
          <motion.div
            key={achievement.title}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.4 + index * 0.1 }}
            className="bg-stellar-lightNavy/30 border border-stellar-lightNavy rounded-xl p-5 text-left"
          >
            <div className="flex items-start">
              <div className="mr-3 mt-0.5">
                {achievement.icon}
              </div>
              <div>
                <h3 className="font-semibold text-white mb-1">{achievement.title}</h3>
                <p className="text-stellar-slate text-sm">{achievement.description}</p>
              </div>
            </div>
          </motion.div>
        ))}
      </div>

      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.8 }}
        className="bg-gradient-to-r from-stellar-lightNavy/30 to-stellar-darkNavy/30 border border-stellar-lightNavy rounded-xl p-6 mb-8"
      >
        <h3 className="text-xl font-semibold text-white mb-3">What's Next?</h3>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-left">
          <div className="bg-stellar-darkNavy/50 p-4 rounded-lg">
            <h4 className="font-medium text-gold-400 mb-2">Explore</h4>
            <p className="text-stellar-slate text-sm">
              Browse guilds and discover communities that align with your interests
            </p>
          </div>
          <div className="bg-stellar-darkNavy/50 p-4 rounded-lg">
            <h4 className="font-medium text-gold-400 mb-2">Contribute</h4>
            <p className="text-stellar-slate text-sm">
              Find and complete bounties to earn rewards and build your reputation
            </p>
          </div>
          <div className="bg-stellar-darkNavy/50 p-4 rounded-lg">
            <h4 className="font-medium text-gold-400 mb-2">Grow</h4>
            <p className="text-stellar-slate text-sm">
              Participate in governance and shape the future of the platform
            </p>
          </div>
        </div>
      </motion.div>

      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 1 }}
        className="flex flex-col sm:flex-row gap-4 justify-center"
      >
        <Button 
          onClick={handleFinish}
          className="bg-gradient-to-r from-emerald-500 to-emerald-600 hover:from-emerald-400 hover:to-emerald-500 text-white px-8 py-3 text-lg font-semibold rounded-xl"
        >
          Enter Stellar Guilds
        </Button>
        
        <Button 
          onClick={handleRestart}
          variant="outline"
          className="border-stellar-lightNavy text-stellar-slate hover:bg-stellar-lightNavy px-8 py-3 text-lg font-semibold rounded-xl"
        >
          Restart Onboarding
        </Button>
      </motion.div>
    </motion.div>
  );
};

export default CompletionStep;