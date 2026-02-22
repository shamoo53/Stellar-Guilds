'use client';

import React from 'react';
import { motion } from 'framer-motion';
import { Sparkles, Users, Trophy, Shield } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { useOnboardingStore } from '@/store/onboardingStore';

const WelcomeStep = () => {
  const { nextStep } = useOnboardingStore();

  const features = [
    {
      icon: <Users className="w-6 h-6" />,
      title: "Guild Management",
      description: "Create, join, and manage decentralized guilds"
    },
    {
      icon: <Trophy className="w-6 h-6" />,
      title: "Bounty System",
      description: "Participate in bounty programs and earn rewards"
    },
    {
      icon: <Shield className="w-6 h-6" />,
      title: "Secure Wallet",
      description: "Integrated Stellar wallet for secure transactions"
    },
    {
      icon: <Sparkles className="w-6 h-6" />,
      title: "Reputation System",
      description: "Build and showcase your reputation score"
    }
  ];

  return (
    <motion.div 
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -20 }}
      className="max-w-4xl mx-auto text-center"
    >
      <motion.div
        initial={{ scale: 0.8, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ delay: 0.2 }}
        className="mb-8"
      >
        <div className="w-20 h-20 bg-gradient-to-r from-gold-500 to-gold-600 rounded-full flex items-center justify-center mx-auto mb-6">
          <Sparkles className="w-10 h-10 text-stellar-navy" />
        </div>
        <h1 className="text-4xl md:text-5xl font-bold bg-gradient-to-r from-gold-400 to-gold-600 bg-clip-text text-transparent mb-4">
          Welcome to Stellar Guilds
        </h1>
        <p className="text-xl text-stellar-slate max-w-2xl mx-auto">
          Your gateway to decentralized collaboration and reward systems on the Stellar network
        </p>
      </motion.div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-12">
        {features.map((feature, index) => (
          <motion.div
            key={feature.title}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.3 + index * 0.1 }}
            className="bg-stellar-lightNavy/50 border border-stellar-lightNavy rounded-xl p-6 text-left hover:bg-stellar-lightNavy/70 transition-all duration-300"
          >
            <div className="w-12 h-12 bg-stellar-darkNavy rounded-lg flex items-center justify-center mb-4 text-gold-400">
              {feature.icon}
            </div>
            <h3 className="text-lg font-semibold text-white mb-2">{feature.title}</h3>
            <p className="text-stellar-slate">{feature.description}</p>
          </motion.div>
        ))}
      </div>

      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.8 }}
        className="bg-gradient-to-r from-stellar-lightNavy/30 to-stellar-darkNavy/30 border border-stellar-lightNavy rounded-xl p-6 mb-8"
      >
        <h3 className="text-xl font-semibold text-white mb-2">What to Expect</h3>
        <ul className="text-stellar-slate space-y-2 text-left max-w-lg mx-auto">
          <li className="flex items-start">
            <span className="text-gold-400 mr-2">•</span>
            <span>Secure wallet setup for Stellar network interactions</span>
          </li>
          <li className="flex items-start">
            <span className="text-gold-400 mr-2">•</span>
            <span>Profile creation to showcase your skills and reputation</span>
          </li>
          <li className="flex items-start">
            <span className="text-gold-400 mr-2">•</span>
            <span>Join or create guilds to collaborate on projects</span>
          </li>
          <li className="flex items-start">
            <span className="text-gold-400 mr-2">•</span>
            <span>Discover and participate in bounty programs</span>
          </li>
        </ul>
      </motion.div>

      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 1 }}
      >
        <Button 
          onClick={nextStep}
          className="bg-gradient-to-r from-gold-500 to-gold-600 hover:from-gold-400 hover:to-gold-500 text-stellar-navy px-8 py-3 text-lg font-semibold rounded-xl"
        >
          Get Started
          <svg xmlns="http://www.w3.org/2000/svg" className="ml-2 h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
            <path fillRule="evenodd" d="M10.293 5.293a1 1 0 011.414 0l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414-1.414L12.586 11H5a1 1 0 110-2h7.586l-2.293-2.293a1 1 0 010-1.414z" clipRule="evenodd" />
          </svg>
        </Button>
      </motion.div>
    </motion.div>
  );
};

export default WelcomeStep;