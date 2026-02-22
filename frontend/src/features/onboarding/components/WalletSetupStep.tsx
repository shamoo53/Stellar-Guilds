'use client';

import React, { useState } from 'react';
import { motion } from 'framer-motion';
import { Wallet, Info, AlertCircle, CheckCircle } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { Card } from '@/components/ui/Card';
import { useWallet } from '@/hooks/useWallet';
import { useOnboardingStore } from '@/store/onboardingStore';
import { WalletProvider } from '@/lib/wallet/types';

const WalletSetupStep = () => {
  const { connect, isConnected, isConnecting, publicKey, provider, disconnect } = useWallet();
  const { nextStep, prevStep } = useOnboardingStore();
  const [showHelp, setShowHelp] = useState(false);

  const handleConnect = async (walletProvider: 'freighter' | 'xumm') => {
    try {
      await connect(walletProvider as WalletProvider);
    } catch (error) {
      console.error('Wallet connection failed:', error);
    }
  };

  const handleNext = () => {
    if (isConnected) {
      nextStep();
    }
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
          <Wallet className="w-8 h-8 text-stellar-navy" />
        </div>
        <h2 className="text-3xl font-bold text-white mb-2">Connect Your Wallet</h2>
        <p className="text-stellar-slate max-w-md mx-auto">
          Securely connect your Stellar wallet to interact with the platform
        </p>
      </div>

      {!isConnected ? (
        <>
          <div className="space-y-4 mb-8">
            <Card className="bg-stellar-lightNavy/30 border border-stellar-lightNavy p-6">
              <div className="flex items-start">
                <Info className="w-5 h-5 text-gold-400 mt-0.5 mr-3 flex-shrink-0" />
                <div>
                  <h3 className="font-semibold text-white mb-1">Why do I need a wallet?</h3>
                  <p className="text-stellar-slate text-sm">
                    A Stellar wallet is required to interact with the blockchain, claim bounties, 
                    participate in governance, and manage your reputation score.
                  </p>
                </div>
              </div>
            </Card>

            <Card className="bg-stellar-lightNavy/30 border border-stellar-lightNavy p-6">
              <div className="flex items-start">
                <AlertCircle className="w-5 h-5 text-blue-400 mt-0.5 mr-3 flex-shrink-0" />
                <div>
                  <h3 className="font-semibold text-white mb-1">Getting Started</h3>
                  <p className="text-stellar-slate text-sm">
                    Download a Stellar-compatible wallet like Freighter (browser extension) 
                    or XUMM (mobile app) before connecting.
                  </p>
                </div>
              </div>
            </Card>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-8">
            <motion.div
              whileHover={{ scale: 1.02 }}
              className="border border-stellar-lightNavy rounded-xl p-6 bg-stellar-lightNavy/20 hover:bg-stellar-lightNavy/40 transition-all cursor-pointer"
              onClick={() => handleConnect('freighter')}
            >
              <div className="flex items-center mb-3">
                <div className="w-10 h-10 bg-gradient-to-r from-blue-500 to-indigo-600 rounded-lg flex items-center justify-center mr-3">
                  <Wallet className="w-5 h-5 text-white" />
                </div>
                <h3 className="font-semibold text-white">Freighter</h3>
              </div>
              <p className="text-stellar-slate text-sm mb-4">
                Browser extension wallet for desktop users
              </p>
              <Button 
                variant="outline" 
                className="w-full border-gold-500 text-gold-400 hover:bg-gold-500/10"
                onClick={(e) => {
                  e.stopPropagation();
                  window.open('https://chrome.google.com/webstore/detail/freighter/abcgjdfhmabhnnkhafpidfibcaofndci', '_blank');
                }}
              >
                Install Extension
              </Button>
            </motion.div>

            <motion.div
              whileHover={{ scale: 1.02 }}
              className="border border-stellar-lightNavy rounded-xl p-6 bg-stellar-lightNavy/20 hover:bg-stellar-lightNavy/40 transition-all cursor-pointer"
              onClick={() => handleConnect('xumm')}
            >
              <div className="flex items-center mb-3">
                <div className="w-10 h-10 bg-gradient-to-r from-green-500 to-emerald-600 rounded-lg flex items-center justify-center mr-3">
                  <Wallet className="w-5 h-5 text-white" />
                </div>
                <h3 className="font-semibold text-white">XUMM</h3>
              </div>
              <p className="text-stellar-slate text-sm mb-4">
                Mobile wallet with QR code pairing
              </p>
              <Button 
                variant="outline" 
                className="w-full border-gold-500 text-gold-400 hover:bg-gold-500/10"
                onClick={(e) => {
                  e.stopPropagation();
                  window.open('https://xumm.app/', '_blank');
                }}
              >
                Download App
              </Button>
            </motion.div>
          </div>
        </>
      ) : (
        <motion.div
          initial={{ scale: 0.9, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          className="bg-gradient-to-r from-emerald-500/10 to-emerald-600/10 border border-emerald-500/30 rounded-xl p-6 mb-8"
        >
          <div className="flex items-center">
            <CheckCircle className="w-6 h-6 text-emerald-400 mr-3" />
            <div>
              <h3 className="font-semibold text-white">Wallet Connected Successfully!</h3>
              <p className="text-stellar-slate text-sm">
                {provider === 'freighter' ? 'Freighter' : 'XUMM'} wallet connected
              </p>
              <p className="font-mono text-xs text-stellar-slate mt-1 truncate">
                {publicKey}
              </p>
            </div>
          </div>
        </motion.div>
      )}

      <div className="flex justify-between">
        <Button 
          variant="outline" 
          onClick={prevStep}
          className="border-stellar-lightNavy text-stellar-slate hover:bg-stellar-lightNavy"
        >
          Back
        </Button>
        
        <Button 
          onClick={handleNext}
          disabled={!isConnected || isConnecting}
          className={`${
            isConnected 
              ? 'bg-gradient-to-r from-gold-500 to-gold-600 hover:from-gold-400 hover:to-gold-500 text-stellar-navy' 
              : 'bg-stellar-lightNavy text-stellar-slate cursor-not-allowed'
          } px-6 py-2 rounded-lg`}
        >
          {isConnecting ? 'Connecting...' : isConnected ? 'Continue' : 'Connect First'}
        </Button>
      </div>
    </motion.div>
  );
};

export default WalletSetupStep;