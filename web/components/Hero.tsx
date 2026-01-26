import React from "react";
import { GithubIcon, DownloadIcon, LogoIcon } from "./Icons";

const Hero: React.FC = () => {
  return (
    <div className="flex flex-col items-center justify-center min-h-[80vh] px-4 text-center max-w-5xl mx-auto pt-20 pb-32">
      <div className="mb-12 relative animate-fade-in-up">
        {/* Glow effect behind logo */}
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-48 h-48 bg-gruvbox-orange/20 blur-3xl rounded-full pointer-events-none"></div>
        <LogoIcon className="w-40 h-40 md:w-56 md:h-56 drop-shadow-2xl object-contain relative z-10 text-gruvbox-orangeBright" />
      </div>

      <h1 className="text-5xl md:text-7xl font-bold mb-6 font-digital tracking-tight text-gruvbox-fg0">
        Typesafe
      </h1>

      <p className="text-xl md:text-2xl text-gruvbox-gray max-w-2xl mb-12 font-light leading-relaxed">
        An intelligent, lightweight{" "}
        <span className="text-gruvbox-orangeBright font-semibold">Rust</span>{" "}
        based LaTeX editor designed for privacy, reliability, and speed.
      </p>

      <div className="flex flex-col sm:flex-row gap-4 w-full sm:w-auto">
        <a
          href="https://github.com/lylebromfield/typesafe/releases/latest/download/typesafe-alpha.zip"
          className="group relative inline-flex items-center justify-center px-8 py-4 bg-gruvbox-orangeBright text-gruvbox-bgHard font-bold rounded-lg text-lg overflow-hidden transition-transform active:scale-95 hover:bg-gruvbox-orange shadow-lg hover:shadow-gruvbox-orange/20"
        >
          <span className="absolute w-0 h-0 transition-all duration-500 ease-out bg-white rounded-full group-hover:w-56 group-hover:h-56 opacity-10"></span>
          <DownloadIcon className="w-6 h-6 mr-2" />
          <span>Download Alpha</span>
        </a>

        <a
          href="https://github.com/lylebromfield/typesafe"
          target="_blank"
          rel="noopener noreferrer"
          className="inline-flex items-center justify-center px-8 py-4 bg-gruvbox-bgSoft text-gruvbox-fg font-bold rounded-lg text-lg border border-gruvbox-gray/30 hover:border-gruvbox-orangeBright hover:text-gruvbox-orangeBright transition-all active:scale-95"
        >
          <GithubIcon className="w-6 h-6 mr-2" />
          <span>View Source</span>
        </a>
      </div>

      <div className="mt-8 text-sm text-gruvbox-gray font-mono">
        v0.3.0 • macOS, Linux, Windows
      </div>
    </div>
  );
};

export default Hero;
