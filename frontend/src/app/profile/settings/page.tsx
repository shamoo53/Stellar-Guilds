"use client";

import React, { useState } from "react";
import { mockUser } from "@/features/profile/mockData";
import { Save, User, Link as LinkIcon, Image as ImageIcon, ArrowLeft } from "lucide-react";
import Link from "next/link";

export default function SettingsPage() {
  const [formData, setFormData] = useState({
    displayName: mockUser.displayName,
    bio: mockUser.bio,
    avatarUrl: mockUser.avatarUrl,
    twitter: "https://twitter.com/tybravo",
    github: "https://github.com/tybravo",
  });

  const [isSaving, setIsSaving] = useState(false);

  const handleChange = (
    e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>
  ) => {
    const { name, value } = e.target;
    setFormData((prev) => ({ ...prev, [name]: value }));
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setIsSaving(true);
    // Simulate API call
    setTimeout(() => {
      setIsSaving(false);
      alert("Profile updated (UI simulation only)");
    }, 1000);
  };

  return (
    <main className="min-h-screen bg-slate-50 pb-20 pt-10 px-4 sm:px-6 lg:px-8">
      <div className="mx-auto max-w-2xl">
        <div className="mb-6 flex items-center gap-4">
          <Link
            href={`/profile/${mockUser.address}`}
            className="rounded-full bg-white p-2 text-gray-500 shadow-sm transition-colors hover:text-gray-900"
          >
            <ArrowLeft className="h-5 w-5" />
          </Link>
          <h1 className="text-2xl font-bold text-gray-900">Profile Settings</h1>
        </div>

        <div className="rounded-2xl bg-white p-6 shadow-sm border border-gray-100 sm:p-8">
          <form onSubmit={handleSubmit} className="space-y-6">
            
            {/* Display Name */}
            <div>
              <label
                htmlFor="displayName"
                className="mb-2 block text-sm font-medium text-gray-900"
              >
                Display Name
              </label>
              <div className="relative">
                <div className="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3">
                  <User className="h-5 w-5 text-gray-400" />
                </div>
                <input
                  type="text"
                  name="displayName"
                  id="displayName"
                  value={formData.displayName}
                  onChange={handleChange}
                  className="block w-full rounded-lg border border-gray-300 bg-gray-50 p-2.5 pl-10 text-gray-900 focus:border-blue-500 focus:ring-blue-500"
                  placeholder="Your display name"
                />
              </div>
            </div>

            {/* Bio */}
            <div>
              <label
                htmlFor="bio"
                className="mb-2 block text-sm font-medium text-gray-900"
              >
                Bio
              </label>
              <textarea
                name="bio"
                id="bio"
                rows={4}
                maxLength={250}
                value={formData.bio}
                onChange={handleChange}
                className="block w-full rounded-lg border border-gray-300 bg-gray-50 p-2.5 text-gray-900 focus:border-blue-500 focus:ring-blue-500"
                placeholder="Tell us about yourself..."
              />
            </div>

            {/* Avatar URL */}
            <div>
              <label
                htmlFor="avatarUrl"
                className="mb-2 block text-sm font-medium text-gray-900"
              >
                Avatar URL
              </label>
              <div className="relative">
                <div className="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3">
                  <ImageIcon className="h-5 w-5 text-gray-400" />
                </div>
                <input
                  type="text"
                  name="avatarUrl"
                  id="avatarUrl"
                  value={formData.avatarUrl}
                  onChange={handleChange}
                  className="block w-full rounded-lg border border-gray-300 bg-gray-50 p-2.5 pl-10 text-gray-900 focus:border-blue-500 focus:ring-blue-500"
                  placeholder="https://..."
                />
              </div>
              <p className="mt-1 text-xs text-gray-500">
                Preview:
              </p>
              <div className="mt-2 h-12 w-12 overflow-hidden rounded-full border border-gray-200">
                <img src={formData.avatarUrl} alt="Avatar preview" className="h-full w-full object-cover" />
              </div>
            </div>

            {/* Social Links */}
            <div className="border-t border-gray-100 pt-6">
              <h3 className="mb-4 text-lg font-medium text-gray-900">
                Social Links
              </h3>
              
              <div className="grid gap-4">
                <div>
                  <label
                    htmlFor="twitter"
                    className="mb-2 block text-sm font-medium text-gray-900"
                  >
                    Twitter / X
                  </label>
                  <div className="relative">
                    <div className="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3">
                      <LinkIcon className="h-5 w-5 text-gray-400" />
                    </div>
                    <input
                      type="text"
                      name="twitter"
                      id="twitter"
                      value={formData.twitter}
                      onChange={handleChange}
                      className="block w-full rounded-lg border border-gray-300 bg-gray-50 p-2.5 pl-10 text-gray-900 focus:border-blue-500 focus:ring-blue-500"
                      placeholder="https://twitter.com/..."
                    />
                  </div>
                </div>

                <div>
                  <label
                    htmlFor="github"
                    className="mb-2 block text-sm font-medium text-gray-900"
                  >
                    GitHub
                  </label>
                  <div className="relative">
                    <div className="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3">
                      <LinkIcon className="h-5 w-5 text-gray-400" />
                    </div>
                    <input
                      type="text"
                      name="github"
                      id="github"
                      value={formData.github}
                      onChange={handleChange}
                      className="block w-full rounded-lg border border-gray-300 bg-gray-50 p-2.5 pl-10 text-gray-900 focus:border-blue-500 focus:ring-blue-500"
                      placeholder="https://github.com/..."
                    />
                  </div>
                </div>
              </div>
            </div>

            {/* Submit Button */}
            <div className="flex justify-end pt-4">
              <button
                type="submit"
                disabled={isSaving}
                className="flex items-center gap-2 rounded-lg bg-blue-600 px-5 py-2.5 text-center text-sm font-medium text-white hover:bg-blue-700 focus:outline-none focus:ring-4 focus:ring-blue-300 disabled:cursor-not-allowed disabled:bg-blue-400"
              >
                <Save className="h-4 w-4" />
                {isSaving ? "Saving..." : "Save Changes"}
              </button>
            </div>
          </form>
        </div>
      </div>
    </main>
  );
}
