'use client';

import React from 'react';
import { motion } from 'framer-motion';
import { Users, Plus, Search, Settings } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { Card } from '@/components/ui/Card';
import { useOnboardingStore } from '@/store/onboardingStore';

const GuildsIntroductionStep = () => {
  const { nextStep, prevStep } = useOnboardingStore();

  const guildFeatures = [
    {
      icon: <Users className="w-6 h-6" />,
      title: "Join Existing Guilds",
      description: "Find and join guilds that match your interests and skills"
    },
    {
      icon: <Plus className="w-6 h-6" />,
      title: "Create Your Own",
      description: "Start your own guild to bring people together around shared goals"
    },
    {
      icon: <Settings className="w-6 h-6" />,
      title: "Manage Roles",
      description: "Define roles and permissions to organize your guild effectively"
    },
    {
      icon: <Search className="w-6 h-6" />,
      title: "Discover Projects",
      description: "Find interesting projects and bounties within guilds"
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
          <Users className="w-8 h-8 text-stellar-navy" />
        </div>
        <h2 className="text-3xl font-bold text-white mb-2">Guilds & Communities</h2>
        <p className="text-stellar-slate max-w-md mx-auto">
          Join or create guilds to collaborate on projects and grow together
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8">
        {guildFeatures.map((feature, index) => (
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
        <h3 className="text-xl font-semibold text-white mb-3">How Guilds Work</h3>
        <div className="space-y-3">
          <div className="flex items-start">
            <div className="w-6 h-6 rounded-full bg-gold-500/20 flex items-center justify-center mr-3 flex-shrink-0 mt-0.5">
              <span className="text-xs font-bold text-gold-400">1</span>
            </div>
            <div>
              <h4 className="font-medium text-white">Discover or Create</h4>
              <p className="text-stellar-slate text-sm">
                Browse existing guilds or start your own with a clear mission and objectives
              </p>
            </div>
          </div>
          <div className="flex items-start">
            <div className="w-6 h-6 rounded-full bg-gold-500/20 flex items-center justify-center mr-3 flex-shrink-0 mt-0.5">
              <span className="text-xs font-bold text-gold-400">2</span>
            </div>
            <div>
              <h4 className="font-medium text-white">Collaborate</h4>
              <p className="text-stellar-slate text-sm">
                Work together on projects, share resources, and contribute to common goals
              </p>
            </div>
          </div>
          <div className="flex items-start">
            <div className="w-6 h-6 rounded-full bg-gold-500/20 flex items-center justify-center mr-3 flex-shrink-0 mt-0.5">
              <span className="text-xs font-bold text-gold-400">3</span>
            </div>
            <div>
              <h4 className="font-medium text-white">Earn Together</h4>
              <p className="text-stellar-slate text-sm">
                Participate in bounty programs and governance to earn rewards collectively
              </p>
            </div>
          </div>
        </div>
      </Card>

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

export default GuildsIntroductionStep;