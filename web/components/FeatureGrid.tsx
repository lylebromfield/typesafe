import React from 'react';
import { FastIcon, ShieldIcon, CodeIcon } from './Icons';

interface FeatureCardProps {
  icon: React.ReactNode;
  title: string;
  description: string;
}

const FeatureCard: React.FC<FeatureCardProps> = ({ icon, title, description }) => (
  <div className="p-8 rounded-xl bg-gruvbox-bgSoft border border-gruvbox-bgHard hover:border-gruvbox-orange/50 transition-colors duration-300 group">
    <div className="w-12 h-12 mb-6 text-gruvbox-gray group-hover:text-gruvbox-orangeBright transition-colors duration-300">
      {icon}
    </div>
    <h3 className="text-2xl font-bold mb-4 text-gruvbox-fg0 font-mono">{title}</h3>
    <p className="text-gruvbox-fg/80 leading-relaxed">{description}</p>
  </div>
);

const FeatureGrid: React.FC = () => {
  return (
    <section id="features" className="py-24 px-4 bg-gruvbox-bgHard/50">
      <div className="max-w-6xl mx-auto">
        <h2 className="text-3xl md:text-4xl font-bold mb-16 text-center text-gruvbox-fg0 font-mono">
          Why Typesafe?
        </h2>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
          <FeatureCard
            icon={<FastIcon className="w-full h-full" />}
            title="Blazing Fast"
            description="Built with Rust to deliver instant feedback. No more waiting for compilation. Live preview updates as you type."
          />
          <FeatureCard
            icon={<ShieldIcon className="w-full h-full" />}
            title="Memory Safe"
            description="Leveraging Rust's ownership model to ensure a crash-free experience. Say goodbye to segmentation faults."
          />
          <FeatureCard
            icon={<CodeIcon className="w-full h-full" />}
            title="Modern Tooling"
            description="Integrated package management, intelligent autocomplete, offline spell-checking, and SyncTeX support out of the box."
          />
        </div>
      </div>
    </section>
  );
};

export default FeatureGrid;
