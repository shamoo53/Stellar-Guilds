'use client';

import React, { useState } from 'react';
import { motion } from 'framer-motion';
import { User, Camera, Briefcase, Globe, Hash } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Textarea } from '@/components/ui/Textarea';
import { Card } from '@/components/ui/Card';
import { useOnboardingStore } from '@/store/onboardingStore';

const ProfileSetupStep = () => {
  const { nextStep, prevStep } = useOnboardingStore();
  const [formData, setFormData] = useState({
    username: '',
    displayName: '',
    bio: '',
    skills: '',
    website: ''
  });

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => {
    const { name, value } = e.target;
    setFormData(prev => ({ ...prev, [name]: value }));
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    // In a real app, this would save to backend
    console.log('Profile data saved:', formData);
    nextStep();
  };

  return (
    <motion.div 
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -20 }}
      className="max-w-2xl mx-auto"
    >
      <div className="text-center mb-8">
        <div className="w-16 h-16 bg-gradient-to-r from-gold-500 to-gold-600 rounded-full flex items-center justify-center mx-auto mb-4">
          <User className="w-8 h-8 text-stellar-navy" />
        </div>
        <h2 className="text-3xl font-bold text-white mb-2">Create Your Profile</h2>
        <p className="text-stellar-slate max-w-md mx-auto">
          Set up your profile to showcase your skills and connect with the community
        </p>
      </div>

      <form onSubmit={handleSubmit}>
        <div className="space-y-6">
          <Card className="bg-stellar-lightNavy/30 border border-stellar-lightNavy p-6">
            <div className="flex items-center mb-4">
              <Camera className="w-5 h-5 text-gold-400 mr-2" />
              <h3 className="font-semibold text-white">Profile Picture</h3>
            </div>
            <div className="flex items-center space-x-4">
              <div className="w-16 h-16 bg-stellar-darkNavy rounded-full flex items-center justify-center border-2 border-dashed border-stellar-lightNavy">
                <User className="w-6 h-6 text-stellar-slate" />
              </div>
              <div>
                <Button type="button" variant="outline" className="border-stellar-lightNavy text-stellar-slate hover:bg-stellar-lightNavy">
                  Upload Photo
                </Button>
                <p className="text-xs text-stellar-slate mt-1">JPG, PNG, or GIF (Max 2MB)</p>
              </div>
            </div>
          </Card>

          <Card className="bg-stellar-lightNavy/30 border border-stellar-lightNavy p-6">
            <div className="space-y-4">
              <div>
                <label htmlFor="username" className="block text-sm font-medium text-stellar-slate mb-1">
                  Username *
                </label>
                <div className="relative">
                  <Hash className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-stellar-slate" />
                  <Input
                    id="username"
                    name="username"
                    value={formData.username}
                    onChange={handleChange}
                    placeholder="your_unique_username"
                    className="pl-10 bg-stellar-darkNavy border-stellar-lightNavy text-white placeholder-stellar-slate"
                    required
                  />
                </div>
                <p className="text-xs text-stellar-slate mt-1">This will be your public identifier</p>
              </div>

              <div>
                <label htmlFor="displayName" className="block text-sm font-medium text-stellar-slate mb-1">
                  Display Name
                </label>
                <div className="relative">
                  <User className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-stellar-slate" />
                  <Input
                    id="displayName"
                    name="displayName"
                    value={formData.displayName}
                    onChange={handleChange}
                    placeholder="John Doe"
                    className="pl-10 bg-stellar-darkNavy border-stellar-lightNavy text-white placeholder-stellar-slate"
                  />
                </div>
              </div>

              <div>
                <label htmlFor="bio" className="block text-sm font-medium text-stellar-slate mb-1">
                  Bio
                </label>
                <Textarea
                  id="bio"
                  name="bio"
                  value={formData.bio}
                  onChange={handleChange}
                  placeholder="Tell us about yourself..."
                  rows={4}
                  className="bg-stellar-darkNavy border-stellar-lightNavy text-white placeholder-stellar-slate"
                />
              </div>

              <div>
                <label htmlFor="skills" className="block text-sm font-medium text-stellar-slate mb-1">
                  Skills
                </label>
                <div className="relative">
                  <Briefcase className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-stellar-slate" />
                  <Input
                    id="skills"
                    name="skills"
                    value={formData.skills}
                    onChange={handleChange}
                    placeholder="Web development, Blockchain, Design..."
                    className="pl-10 bg-stellar-darkNavy border-stellar-lightNavy text-white placeholder-stellar-slate"
                  />
                </div>
              </div>

              <div>
                <label htmlFor="website" className="block text-sm font-medium text-stellar-slate mb-1">
                  Website
                </label>
                <div className="relative">
                  <Globe className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-stellar-slate" />
                  <Input
                    id="website"
                    name="website"
                    type="url"
                    value={formData.website}
                    onChange={handleChange}
                    placeholder="https://your-website.com"
                    className="pl-10 bg-stellar-darkNavy border-stellar-lightNavy text-white placeholder-stellar-slate"
                  />
                </div>
              </div>
            </div>
          </Card>
        </div>

        <div className="flex justify-between mt-8">
          <Button 
            variant="outline" 
            onClick={prevStep}
            type="button"
            className="border-stellar-lightNavy text-stellar-slate hover:bg-stellar-lightNavy"
          >
            Back
          </Button>
          
          <Button 
            type="submit"
            className="bg-gradient-to-r from-gold-500 to-gold-600 hover:from-gold-400 hover:to-gold-500 text-stellar-navy px-6 py-2 rounded-lg"
          >
            Continue
          </Button>
        </div>
      </form>
    </motion.div>
  );
};

export default ProfileSetupStep;