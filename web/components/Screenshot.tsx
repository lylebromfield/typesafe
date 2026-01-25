import React from "react";

const Screenshot: React.FC = () => {
  return (
    <section className="py-20 bg-gruvbox-bgHard relative overflow-hidden">
      <div className="container mx-auto px-4 max-w-6xl relative z-10">
        <div className="text-center mb-16">
          <h2 className="text-3xl md:text-5xl font-bold mb-6 text-gruvbox-fg0 font-digital">
            Distraction-Free <span className="text-gruvbox-aqua">Writing</span>
          </h2>
          <p className="text-xl text-gruvbox-gray max-w-2xl mx-auto leading-relaxed">
            Experience a clean, modern interface that gets out of your way. With
            instant live preview, integrated Vim bindings, and smart
            autocompletion, Typesafe lets you focus on your content, not the
            compiler.
          </p>
        </div>

        <div className="relative group rounded-xl bg-gruvbox-bgSoft p-2 sm:p-4 shadow-2xl border border-gruvbox-gray/20">
          <div className="absolute inset-0 bg-gradient-to-tr from-gruvbox-orange/10 to-gruvbox-aqua/10 opacity-50 pointer-events-none rounded-xl"></div>

          {/* Window Controls Decoration */}
          <div className="flex gap-2 mb-4 px-2 opacity-70">
            <div className="w-3 h-3 rounded-full bg-gruvbox-red"></div>
            <div className="w-3 h-3 rounded-full bg-gruvbox-yellow"></div>
            <div className="w-3 h-3 rounded-full bg-gruvbox-green"></div>
          </div>

          <img
            src="/screenshot.png"
            alt="Typesafe Editor Interface"
            className="w-full rounded-lg shadow-inner border border-gruvbox-bg"
            loading="lazy"
          />
        </div>
      </div>

      {/* Background decorations */}
      <div className="absolute top-0 left-0 w-full h-full overflow-hidden pointer-events-none opacity-20">
          <div className="absolute top-[10%] left-[-5%] w-96 h-96 bg-gruvbox-purple/20 blur-3xl rounded-full"></div>
          <div className="absolute bottom-[10%] right-[-5%] w-96 h-96 bg-gruvbox-blue/20 blur-3xl rounded-full"></div>
      </div>
    </section>
  );
};

export default Screenshot;
