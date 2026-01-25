import React from 'react';
import { LogoIcon } from './Icons';

const Navbar: React.FC = () => {
  return (
    <nav className="w-full py-6 px-4 sm:px-8 flex justify-between items-center sticky top-0 z-50 bg-gruvbox-bg/95 backdrop-blur-sm border-b border-gruvbox-bgSoft">
      <div className="flex items-center gap-3">
        <div className="h-8 w-8">
           <LogoIcon className="w-full h-full object-contain text-gruvbox-orangeBright" />
        </div>
        <span className="font-digital font-bold text-xl tracking-tight text-gruvbox-fg">Typesafe</span>
      </div>
      <div className="flex items-center gap-6">
        <a href="#features" className="hidden sm:block text-gruvbox-fg hover:text-gruvbox-orangeBright transition-colors font-medium">Features</a>
        <a href="https://github.com/lylebromfield/typesafe" target="_blank" rel="noopener noreferrer" className="text-gruvbox-fg hover:text-gruvbox-orangeBright transition-colors font-medium">GitHub</a>
      </div>
    </nav>
  );
};

export default Navbar;