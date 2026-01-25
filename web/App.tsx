import React from 'react';
import Navbar from './components/Navbar';
import Hero from './components/Hero';
import FeatureGrid from './components/FeatureGrid';
import Footer from './components/Footer';

const App: React.FC = () => {
  return (
    <div className="min-h-screen bg-gruvbox-bg text-gruvbox-fg selection:bg-gruvbox-orange selection:text-gruvbox-bgHard font-sans">
      <Navbar />
      <main>
        <Hero />
        <FeatureGrid />
      </main>
      <Footer />
    </div>
  );
};

export default App;