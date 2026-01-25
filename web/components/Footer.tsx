import React from 'react';

const Footer: React.FC = () => {
  return (
    <footer className="py-12 px-4 border-t border-gruvbox-bgSoft text-center bg-gruvbox-bg">
      <div className="max-w-4xl mx-auto flex flex-col items-center gap-6">
        <p className="text-gruvbox-gray text-sm">
          &copy; {new Date().getFullYear()} Typesafe Contributors. Open source under MIT License.
        </p>
        <div className="flex gap-4">
          <a href="#" className="text-gruvbox-gray hover:text-gruvbox-orangeBright transition-colors text-sm">Privacy</a>
          <a href="#" className="text-gruvbox-gray hover:text-gruvbox-orangeBright transition-colors text-sm">Terms</a>
          <a href="https://github.com/lylebromfield/typesafe/issues" className="text-gruvbox-gray hover:text-gruvbox-orangeBright transition-colors text-sm">Report Issue</a>
        </div>
      </div>
    </footer>
  );
};

export default Footer;