'use client';

import React from 'react';
import { motion } from 'framer-motion';
import { Trophy, Coins, Target, Award } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { Card } from '@/components/ui/Card';
import { useOnboardingStore } from '@/store/onboardingStore';

const BountiesIntroductionStep = () => {
  const { nextStep, prevStep } = useOnboardingStore();

  const bountyFeatures = [
    {
      icon: <Target className="w-6 h-6" />,
      title: "Find Opportunities",
      description: "Browse various bounties across different guilds and skill levels"
    },
    {
      icon: <Coins className="w-6 h-6" />,
      title: "Earn Rewards",
      description: "Get paid in XLM or other tokens for completing bounties"
    },
    {
      icon: <Award className="w-6 h-6" />,
      title: "Build Reputation",
      description: "Increase your reputation score by successfully completing bounties"
    },
    {
      icon: <Trophy className="w-6 h-6" />,
      title: "Level Up",
      description: "Unlock higher-value bounties as your reputation grows"
    }
  ];

  return (
    <motion.div 
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -20 }}
      className="max-w-3xl mx-auto"
    >
      <div className="text-center mb-8">
        <div className="w-16 h-16 bg-gradient-to-r from-gold-500 to-gold-600 rounded-full flex items-center justify-center mx-auto mb-4">
          <Trophy className="w-8 h-8 text-stellar-navy" />
        </div>
        <h2 className="text-3xl font-bold text-white mb-2">Bounties & Rewards</h2>
        <p className="text-stellar-slate max-w-md mx-auto">
          Find and complete bounties to earn rewards and build your reputation
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8">
        {bountyFeatures.map((feature, index) => (
          <motion.div
            key={feature.title}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.1 * index }}
            className="bg-stellar-lightNavy/30 border border-stellar-lightNavy rounded-xl p-5 hover:bg-stellar-lightNavy/50 transition-all"
          >
            <div className="w-10 h-10 bg-stellar-darkNavy rounded-lg flex items-center justify-center mb-3 text-gold-400">
              {feature.icon}
            </div>
            <h3 className="font-semibold text-white mb-2">{feature.title}</h3>
            <p className="text-stellar-slate text-sm">{feature.description}</p>
          </motion.div>
        ))}
      </div>

      <Card className="bg-gradient-to-r from-stellar-lightNavy/30 to-stellar-darkNavy/30 border border-stellar-lightNavy p-6 mb-8">
        <h3 className="text-xl font-semibold text-white mb-3">How Bounties Work</h3>
        <div className="space-y-3">
          <div className="flex items-start">
            <div className="w-6 h-6 rounded-full bg-gold-500/20 flex items-center justify-center mr-3 flex-shrink-0 mt-0.5">
              <span className="text-xs font-bold text-gold-400">1</span>
            </div>
            <div>
              <h4 className="font-medium text-white">Discover</h4>
              <p className="text-stellar-slate text-sm">
                Browse available bounties based on your skills and interests
              </p>
            </div>
          </div>
          <div className="flex items-start">
            <div className="w-6 h-6 rounded-full bg-gold-500/20 flex items-center justify-center mr-3 flex-shrink-0 mt-0.5">
              <span className="text-xs font-bold text-gold-400">2</span>
            </div>
            <div>
              <h4 className="font-medium text-white">Apply/Claim</h4>
              <p className="text-stellar-slate text-sm">
                Submit your application or claim a bounty that matches your skills
              </p>
            </div>
          </div>
          <div className="flex items-start">
            <div className="w-6 h-6 rounded-full bg-gold-500/20 flex items-center justify-center mr-3 flex-shrink-0 mt-0.5">
              <span className="text-xs font-bold text-gold-400">3</span>
            </div>
            <div>
              <h4 className="font-medium text-white">Complete</h4>
              <p className="text-stellar-slate text-sm">
                Work on the bounty according to the requirements and timeline
              </p>
            </div>
          </div>
          <div className="flex items-start">
            <div className="w-6 h-6 rounded-full bg-gold-500/20 flex items-center justify-center mr-3 flex-shrink-0 mt-0.5">
              <span className="text-xs font-bold text-gold-400">4</span>
            </div>
            <div>
              <h4 className="font-medium text-white">Get Rewarded</h4>
              <p className="text-stellar-slate text-sm">
                Receive payment and increase your reputation upon successful completion
              </p>
            </div>
          </div>
        </div>
      </Card>

      <div className="bg-stellar-lightNavy/30 border border-stellar-lightNavy rounded-xl p-6 mb-8">
        <div className="flex items-start">
          <div className="w-10 h-10 bg-gradient-to-r from-emerald-500 to-emerald-600 rounded-lg flex items-center justify-center mr-4 flex-shrink-0">
            <Coins className="w-5 h-5 text-white" />
          </div>
          <div>
            <h3 className="font-semibold text-white mb-1">Pro Tip</h3>
            <p className="text-stellar-slate text-sm">
              Start with smaller bounties to build your reputation and gradually take on more challenging ones. 
              Quality work leads to more opportunities and higher rewards!
            </p>
          </div>
        </div>
      </div>

      <div className="flex justify-between">
        <Button 
          variant="outline" 
          onClick={prevStep}
          className="border-stellar-lightNavy text-stellar-slate hover:bg-stellar-lightNavy"
        >
          Back
        </Button>
        
        <Button 
          onClick={nextStep}
          className="bg-gradient-to-r from-gold-500 to-gold-600 hover:from-gold-400 hover:to-gold-500 text-stellar-navy px-6 py-2 rounded-lg"
        >
          Continue
        </Button>
      </div>
    </motion.div>
  );
};

export default BountiesIntroductionStep;