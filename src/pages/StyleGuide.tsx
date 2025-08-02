
import { useTheme } from '../contexts/ThemeContext';
import { AllayLayout } from '../components/common/AllayLayout';
import { ScrollContainer } from '../components/common/ScrollContainer';
import { Dropdown } from '../components/common/Dropdown';
import { ToolTip } from '../components/common/ToolTip';
import { MinecraftMOTD } from '../components/common/MinecraftMOTD';
import { MemorySlider } from '../components/common/MemorySlider';
import { RadioGroup } from '../components/common/RadioGroup';
import { useState } from 'react';
import { Play, Square, Plus, Search, Settings2 } from 'lucide-react';

export const StyleGuide = () => {
  const { theme, setTheme, toggleTheme } = useTheme();
  
  // State for interactive examples
  const [memoryValue, setMemoryValue] = useState(2048);
  const [radioValue, setRadioValue] = useState('option1');

  return (
    <div className="h-screen bg-[var(--color-background)] text-[var(--color-text)] flex flex-col" style={{ fontFamily: '"Noto Sans", sans-serif' }}>
      <AllayLayout title="Style Guide" />
      <ScrollContainer className="flex-1">
      {/* Hero Section */}
      <section className="relative overflow-hidden bg-gradient-to-br from-[var(--color-primary)] to-[var(--color-secondary)] text-white pt-12">
        <div className="absolute inset-0 bg-black/10"></div>
        <div className="relative px-8 py-24">
          <div className="max-w-4xl mx-auto text-center">
            <h1 className="text-6xl font-bold mb-6 tracking-tight">
              Allay Style Guide
            </h1>
            <p className="text-xl font-light mb-8 opacity-90 max-w-2xl mx-auto leading-relaxed">
              Design system and component library for the ultimate Minecraft server management experience
            </p>
            <button 
              onClick={toggleTheme}
              className="inline-flex items-center px-6 py-3 bg-white/20 hover:bg-white/30 backdrop-blur-sm rounded-lg transition-all duration-200 font-medium"
            >
              Switch Theme ({theme === 'allay' ? 'Light → Dark' : theme === 'allay-dark' ? 'Dark → Enderman' : theme === 'enderman' ? 'Enderman → Emerald Obsidian' : theme === 'emerald-obsidian' ? 'Emerald Obsidian → Copper Golem' : 'Copper Golem → Light'})
            </button>
          </div>
        </div>
      </section>

      <div className="max-w-6xl mx-auto px-8 py-16">
        {/* Typography Section */}
        <section className="mb-20">
          <h2 className="text-4xl font-bold mb-8 text-[var(--color-text)]">Typography</h2>
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-12">
            <div>
              <h3 className="text-2xl font-semibold mb-6 text-[var(--color-text-secondary)]">Headings</h3>
              <div className="space-y-4">
                <h1 className="text-5xl font-bold text-[var(--color-text)]">Heading 1</h1>
                <h2 className="text-4xl font-bold text-[var(--color-text)]">Heading 2</h2>
                <h3 className="text-3xl font-semibold text-[var(--color-text)]">Heading 3</h3>
                <h4 className="text-2xl font-semibold text-[var(--color-text)]">Heading 4</h4>
                <h5 className="text-xl font-medium text-[var(--color-text)]">Heading 5</h5>
                <h6 className="text-lg font-medium text-[var(--color-text)]">Heading 6</h6>
              </div>
            </div>
            <div>
              <h3 className="text-2xl font-semibold mb-6 text-[var(--color-text-secondary)]">Body Text</h3>
              <div className="space-y-4">
                <p className="text-lg text-[var(--color-text)]">
                  <strong>Large Text:</strong> This is larger body text used for important content and introductions.
                </p>
                <p className="text-base text-[var(--color-text)]">
                  <strong>Regular Text:</strong> This is the standard body text used throughout the application for most content.
                </p>
                <p className="text-sm text-[var(--color-text-secondary)]">
                  <strong>Small Text:</strong> This is smaller text used for captions, labels, and secondary information.
                </p>
                <p className="text-xs text-[var(--color-text-muted)]">
                  <strong>Tiny Text:</strong> This is the smallest text used for fine print and metadata.
                </p>
              </div>
              <div className="mt-8">
                <h4 className="text-lg font-semibold mb-4 text-[var(--color-text)]">Font Weights</h4>
                <div className="space-y-2">
                  <p className="font-light text-[var(--color-text)]">Light (300)</p>
                  <p className="font-normal text-[var(--color-text)]">Regular (400)</p>
                  <p className="font-medium text-[var(--color-text)]">Medium (500)</p>
                  <p className="font-semibold text-[var(--color-text)]">Semibold (600)</p>
                  <p className="font-bold text-[var(--color-text)]">Bold (700)</p>
                  <p className="font-black text-[var(--color-text)]">Black (900)</p>
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* Color Palette Section */}
        <section className="mb-20">
          <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between mb-8 gap-4">
            <h2 className="text-4xl font-bold text-[var(--color-text)]">Color Palette</h2>
            <div className="sm:w-48">
              <Dropdown
                options={[
                  { value: 'allay', label: 'Allay Light' },
                  { value: 'allay-dark', label: 'Allay Dark' },
                  { value: 'enderman', label: 'Enderman' },
                  { value: 'emerald-obsidian', label: 'Emerald Obsidian' },
                  { value: 'copper-golem', label: 'Copper Golem' }
                ]}
                value={theme}
                onChange={(value) => setTheme(value as 'allay' | 'allay-dark' | 'enderman' | 'emerald-obsidian' | 'copper-golem')}
                placeholder="Select theme..."
                className="min-w-[160px]"
              />
            </div>
          </div>
          
          {/* Primary Colors */}
          <div className="mb-12">
            <h3 className="text-2xl font-semibold mb-6 text-[var(--color-text-secondary)]">Primary Colors</h3>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-6">
              <ColorSwatch color="var(--color-primary)" name="Primary" />
              <ColorSwatch color="var(--color-primary-hover)" name="Primary Hover" />
              <ColorSwatch color="var(--color-secondary)" name="Secondary" />
              <ColorSwatch color="var(--color-secondary-hover)" name="Secondary Hover" />
            </div>
          </div>

          {/* Surface Colors */}
          <div className="mb-12">
            <h3 className="text-2xl font-semibold mb-6 text-[var(--color-text-secondary)]">Surface Colors</h3>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-6">
              <ColorSwatch color="var(--color-background)" name="Background" />
              <ColorSwatch color="var(--color-surface)" name="Surface" />
              <ColorSwatch color="var(--color-surface-hover)" name="Surface Hover" />
              <ColorSwatch color="var(--color-border)" name="Border" />
            </div>
          </div>

          {/* Status Colors */}
          <div className="mb-12">
            <h3 className="text-2xl font-semibold mb-6 text-[var(--color-text-secondary)]">Status Colors</h3>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-6">
              <ColorSwatch color="var(--color-success)" name="Success" />
              <ColorSwatch color="var(--color-warning)" name="Warning" />
              <ColorSwatch color="var(--color-danger)" name="Danger" />
              <ColorSwatch color="var(--color-accent)" name="Accent" />
            </div>
          </div>

          {/* Server Status Colors */}
          <div className="mb-12">
            <h3 className="text-2xl font-semibold mb-6 text-[var(--color-text-secondary)]">Server Status</h3>
            <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
              <StatusBadge status="online" />
              <StatusBadge status="offline" />
              <StatusBadge status="starting" />
              <StatusBadge status="stopping" />
              <StatusBadge status="error" />
            </div>
          </div>

          {/* Mod Loader Colors */}
          <div className="mb-12">
            <h3 className="text-2xl font-semibold mb-6 text-[var(--color-text-secondary)]">Mod Loaders</h3>
            <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-8 gap-4">
              <ModLoaderBadge loader="vanilla" />
              <ModLoaderBadge loader="fabric" />
              <ModLoaderBadge loader="forge" />
              <ModLoaderBadge loader="neoforge" />
              <ModLoaderBadge loader="quilt" />
              <ModLoaderBadge loader="paper" />
              <ModLoaderBadge loader="bukkit" />
              <ModLoaderBadge loader="spigot" />
            </div>
          </div>
        </section>

        {/* Component Examples */}
        <section className="mb-20">
          <h2 className="text-4xl font-bold mb-8 text-[var(--color-text)]">Components</h2>
          
          {/* Buttons */}
          <div className="mb-12">
            <h3 className="text-2xl font-semibold mb-6 text-[var(--color-text-secondary)]">Buttons</h3>
            <div className="flex flex-wrap gap-4">
              <button className="px-6 py-3 bg-[var(--color-primary)] hover:bg-[var(--color-primary-hover)] text-white rounded-lg font-medium transition-colors">
                Primary Button
              </button>
              <button className="px-6 py-3 bg-[var(--color-secondary)] hover:bg-[var(--color-secondary-hover)] text-white rounded-lg font-medium transition-colors">
                Secondary Button
              </button>
              <button className="px-6 py-3 bg-[var(--color-surface)] hover:bg-[var(--color-surface-hover)] text-[var(--color-text)] border border-[var(--color-border)] rounded-lg font-medium transition-colors">
                Outline Button
              </button>
              <button className="px-6 py-3 bg-[var(--color-danger)] hover:bg-[var(--color-danger-hover)] text-white rounded-lg font-medium transition-colors">
                Danger Button
              </button>
            </div>
          </div>

          {/* Cards */}
          <div className="mb-12">
            <h3 className="text-2xl font-semibold mb-6 text-[var(--color-text-secondary)]">Cards</h3>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
              <div className="p-6 bg-[var(--color-surface)] border border-[var(--color-border)] rounded-xl">
                <h4 className="text-xl font-semibold mb-3 text-[var(--color-text)]">Basic Card</h4>
                <p className="text-[var(--color-text-secondary)] mb-4">This is a basic card component with standard styling.</p>
                <button className="px-4 py-2 bg-[var(--color-primary)] text-white rounded-lg text-sm font-medium">
                  Action
                </button>
              </div>
              <div className="p-6 bg-[var(--color-surface)] border border-[var(--color-border)] rounded-xl hover:bg-[var(--color-surface-hover)] transition-colors">
                <h4 className="text-xl font-semibold mb-3 text-[var(--color-text)]">Hover Card</h4>
                <p className="text-[var(--color-text-secondary)] mb-4">This card has hover effects and interactive styling.</p>
                <button className="px-4 py-2 bg-[var(--color-secondary)] text-white rounded-lg text-sm font-medium">
                  Action
                </button>
              </div>
              <div className="p-6 bg-gradient-to-br from-[var(--color-primary)] to-[var(--color-secondary)] text-white rounded-xl">
                <h4 className="text-xl font-semibold mb-3">Gradient Card</h4>
                <p className="opacity-90 mb-4">This card uses the primary gradient background.</p>
                <button className="px-4 py-2 bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white rounded-lg text-sm font-medium transition-colors">
                  Action
                </button>
              </div>
            </div>
          </div>

          {/* Server Cards */}
          <div className="mb-12">
            <h3 className="text-2xl font-semibold mb-6 text-[var(--color-text-secondary)]">Server Cards</h3>
            <div className="space-y-4">
              <ServerCardExample 
                name="Survival World"
                description="A classic survival experience with custom plugins"
                serverType="Paper"
                version="1.21"
                loaderVersion="Paper 1.21"
                isOnline={true}
                playerCount={12}
                maxPlayers={20}
              />
              <ServerCardExample 
                name="Creative Build Server"
                description="Unlimited creativity with WorldEdit and custom tools"
                serverType="Fabric"
                version="1.20.4"
                loaderVersion="Fabric 0.96.11"
                isOnline={false}
                playerCount={0}
                maxPlayers={50}
              />
              <ServerCardExample 
                name="Modded Adventure"
                description="Explore new dimensions with 200+ mods installed"
                serverType="Forge"
                version="1.19.2"
                loaderVersion="Forge 43.3.7"
                isOnline={true}
                playerCount={8}
                maxPlayers={15}
              />
            </div>
          </div>

          {/* ActionBar */}
          <div className="mb-12">
            <h3 className="text-2xl font-semibold mb-6 text-[var(--color-text-secondary)]">Action Bar</h3>
            <div className="bg-[var(--color-surface)] border border-[var(--color-border)] rounded-xl p-6">
              <div className="relative h-16 flex items-center">
                <ActionBarExample />
              </div>
              <p className="text-sm text-[var(--color-text-secondary)] mt-4">
                Interactive action bar with expandable icons and tooltips. Click the arrow to expand/collapse.
              </p>
            </div>
          </div>

          {/* Tooltips */}
          <div className="mb-12">
            <h3 className="text-2xl font-semibold mb-6 text-[var(--color-text-secondary)]">Tooltips</h3>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
              <div className="text-center">
                <ToolTip content="Tooltip on top" position="top">
                  <button className="px-4 py-2 bg-[var(--color-primary)] text-white rounded-lg font-medium">
                    Top
                  </button>
                </ToolTip>
              </div>
              <div className="text-center">
                <ToolTip content="Tooltip on bottom" position="bottom">
                  <button className="px-4 py-2 bg-[var(--color-secondary)] text-white rounded-lg font-medium">
                    Bottom
                  </button>
                </ToolTip>
              </div>
              <div className="text-center">
                <ToolTip content="Tooltip on left" position="left">
                  <button className="px-4 py-2 bg-[var(--color-accent)] text-white rounded-lg font-medium">
                    Left
                  </button>
                </ToolTip>
              </div>
              <div className="text-center">
                <ToolTip content="Tooltip on right" position="right">
                  <button className="px-4 py-2 bg-[var(--color-success)] text-white rounded-lg font-medium">
                    Right
                  </button>
                </ToolTip>
              </div>
            </div>
          </div>

          {/* Minecraft MOTD */}
          <div className="mb-12">
            <h3 className="text-2xl font-semibold mb-6 text-[var(--color-text-secondary)]">Minecraft MOTD</h3>
            <div className="space-y-6">
              <div className="p-4 bg-[var(--color-surface)] border border-[var(--color-border)] rounded-xl">
                <h4 className="text-lg font-medium mb-3 text-[var(--color-text)]">Colorful MOTD</h4>
                <MinecraftMOTD 
                  motd="§6Welcome to §bAllay Server§r! §aJoin us for §cfun §eadventures!"
                  theme={theme === 'allay' ? 'light' : 'dark'}
                />
              </div>
              <div className="p-4 bg-[var(--color-surface)] border border-[var(--color-border)] rounded-xl">
                <h4 className="text-lg font-medium mb-3 text-[var(--color-text)]">Formatted MOTD</h4>
                <MinecraftMOTD 
                  motd="§l§4PvP Server §r§8- §o§9Epic battles await§r §k§a!!!§r"
                  theme={theme === 'allay' ? 'light' : 'dark'}
                />
              </div>
              <div className="p-4 bg-[var(--color-surface)] border border-[var(--color-border)] rounded-xl">
                <h4 className="text-lg font-medium mb-3 text-[var(--color-text)]">Simple MOTD</h4>
                <MinecraftMOTD 
                  motd="A peaceful creative server for building amazing structures"
                  theme={theme === 'allay' ? 'light' : 'dark'}
                />
              </div>
            </div>
          </div>

          {/* Memory Slider */}
          <div className="mb-12">
            <h3 className="text-2xl font-semibold mb-6 text-[var(--color-text-secondary)]">Memory Slider</h3>
            <div className="max-w-2xl">
              <MemorySlider 
                value={memoryValue}
                onChange={setMemoryValue}
                className="p-6 bg-[var(--color-surface)] border border-[var(--color-border)] rounded-xl"
              />
            </div>
          </div>

          {/* Radio Groups */}
          <div className="mb-12">
            <h3 className="text-2xl font-semibold mb-6 text-[var(--color-text-secondary)]">Radio Groups</h3>
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
              <div>
                <h4 className="text-lg font-medium mb-4 text-[var(--color-text)]">Vertical Layout</h4>
                <RadioGroup
                  name="example-vertical"
                  options={[
                    { value: 'option1', label: 'Option 1', description: 'This is the first option' },
                    { value: 'option2', label: 'Option 2', description: 'This is the second option' },
                    { value: 'option3', label: 'Option 3', description: 'This is the third option' }
                  ]}
                  value={radioValue}
                  onChange={setRadioValue}
                  layout="vertical"
                />
              </div>
              <div>
                <h4 className="text-lg font-medium mb-4 text-[var(--color-text)]">Grid Layout</h4>
                <RadioGroup
                  name="example-grid"
                  options={[
                    { value: 'grid1', label: 'Grid 1' },
                    { value: 'grid2', label: 'Grid 2' },
                    { value: 'grid3', label: 'Grid 3' },
                    { value: 'grid4', label: 'Grid 4' }
                  ]}
                  layout="grid"
                />
              </div>
            </div>
          </div>

          {/* Terminal */}
          <div className="mb-12">
            <h3 className="text-2xl font-semibold mb-6 text-[var(--color-text-secondary)]">Terminal</h3>
            <div className="h-96">
              <TerminalExample />
            </div>
          </div>
        </section>
      </div>
      </ScrollContainer>
    </div>
  );
};

// Helper Components
const ColorSwatch = ({ color, name }: { color: string; name: string }) => (
  <div className="text-center">
    <div 
      className="w-full h-20 rounded-lg mb-3 border border-[var(--color-border)]" 
      style={{ backgroundColor: `${color}` }}
    ></div>
    <p className="text-sm font-medium text-[var(--color-text)]">{name}</p>
    <p className="text-xs text-[var(--color-text-muted)] font-mono">{color}</p>
  </div>
);

const StatusBadge = ({ status }: { status: string }) => (
  <div className="text-center">
    <div 
      className={`px-4 py-2 rounded-lg text-sm font-medium capitalize`}
      style={{ 
        backgroundColor: `var(--color-status-${status})`,
        color: `var(--color-status-${status}-font)`
      }}
    >
      {status}
    </div>
  </div>
);

const ModLoaderBadge = ({ loader }: { loader: string }) => (
  <div className="text-center">
    <div 
      className={`px-3 py-2 rounded-lg text-sm font-medium capitalize`}
      style={{ 
        backgroundColor: `var(--color-loader-${loader})`,
        color: `var(--color-loader-font-${loader})`
      }}
    >
      {loader}
    </div>
  </div>
);

const ServerCardExample = ({ 
  name, 
  description, 
  serverType, 
  version, 
  loaderVersion, 
  isOnline, 
  playerCount, 
  maxPlayers 
}: {
  name: string;
  description: string;
  serverType: string;
  version: string;
  loaderVersion: string;
  isOnline: boolean;
  playerCount: number;
  maxPlayers: number;
}) => (
  <div className="bg-[var(--color-background)] rounded-lg shadow-sm border border-[var(--color-border)] p-4 hover:border-[var(--color-border-hover)] transition-colors">
    <div className="flex items-start justify-between">
      <div className="flex items-start space-x-4">
        {/* Server Icon */}
        <div className="w-16 h-16 rounded-lg overflow-hidden bg-[var(--color-surface)] flex items-center justify-center">
          <div 
            className="w-12 h-12 rounded-lg"
            style={{ 
              backgroundColor: isOnline ? 'var(--color-success)' : 'var(--color-text-muted)',
              opacity: isOnline ? 1 : 0.6
            }}
          ></div>
        </div>

        {/* Server Info */}
        <div className="flex-1">
          <h3 className="text-lg font-semibold text-[var(--color-text)] mb-1">{name}</h3>
          <p className="text-[var(--color-text-secondary)] text-sm mb-3 leading-relaxed">
            {description}
          </p>

          {/* Tags */}
          <div className="flex flex-wrap gap-2">
            <span className="px-2 py-1 bg-[var(--color-surface)] text-[var(--color-text-secondary)] text-xs rounded-full">
              {serverType}
            </span>
            <span 
              className="px-2 py-1 text-xs rounded-full"
              style={{ 
                backgroundColor: 'var(--color-version)', 
                color: 'var(--color-version-font)' 
              }}
            >
              {version}
            </span>
            {loaderVersion && (
              <span 
                className="px-2 py-1 text-xs rounded-full"
                style={{ 
                  backgroundColor: `var(--color-loader-${serverType.toLowerCase()})`, 
                  color: `var(--color-loader-font-${serverType.toLowerCase()})` 
                }}
              >
                {loaderVersion}
              </span>
            )}
            {isOnline ? (
              <span 
                className="px-2 py-1 text-xs rounded-full"
                style={{ 
                  backgroundColor: 'var(--color-status-online)', 
                  color: 'var(--color-status-online-font)' 
                }}
              >
                Online
              </span>
            ) : (
              <span 
                className="px-2 py-1 text-xs rounded-full"
                style={{ 
                  backgroundColor: 'var(--color-status-offline)', 
                  color: 'var(--color-status-offline-font)' 
                }}
              >
                Offline
              </span>
            )}
          </div>
        </div>
      </div>

      {/* Controls and Player Count */}
      <div className="flex flex-col items-end space-y-3">
        {/* Start/Stop Button */}
        <button
          className={`flex items-center justify-center gap-2 px-4 py-2 rounded-lg transition-all duration-200 font-medium text-sm hover:scale-105 active:scale-95`}
          style={{
            backgroundColor: isOnline ? 'var(--color-danger)' : 'var(--color-success)',
            color: 'white'
          }}
        >
          <div className="w-4 h-4 flex items-center justify-center">
            {isOnline ? (
              <div className="w-2 h-2 bg-white rounded-sm"></div>
            ) : (
              <div className="w-0 h-0 border-l-[6px] border-l-white border-t-[3px] border-t-transparent border-b-[3px] border-b-transparent ml-0.5"></div>
            )}
          </div>
          {isOnline ? 'Stop' : 'Run'}
        </button>

        {/* Player Count */}
        <div className="text-right">
          <span className="text-lg font-semibold text-[var(--color-text)]">
            {playerCount}/{maxPlayers}
          </span>
          <p className="text-xs text-[var(--color-text-muted)]">
            Players
          </p>
        </div>
      </div>
    </div>
  </div>
);

const ActionBarExample = () => {
  const [isExpanded, setIsExpanded] = useState(true);

  const actionIcons = [
    { icon: Plus, tooltip: "Create" },
    { icon: Search, tooltip: "Search" },
    { icon: Settings2, tooltip: "Settings" }
  ];

  return (
    <div className="flex items-center">
      <div
        className="flex items-center transition-all ease-in-out duration-300"
        style={{ width: isExpanded ? '200px' : '48px' }}
      >
        <div className={`flex items-center space-x-2 ${!isExpanded ? 'overflow-hidden' : ''}`}>
          {actionIcons.map(({ icon: Icon, tooltip }, index) => (
            <div
              key={index}
              className={`transition-opacity ease-out duration-200 ${
                isExpanded ? 'opacity-100' : 'opacity-0 pointer-events-none'
              }`}
              style={{ transitionDelay: isExpanded ? `${index * 50}ms` : '0ms' }}
            >
              <ToolTip content={tooltip} position="bottom" delay={300}>
                <button className="ml-2 p-2 rounded hover:bg-[var(--color-surface-hover)] cursor-pointer text-[var(--color-text-secondary)] hover:text-[var(--color-text)] transition-colors">
                  <Icon size={16} />
                </button>
              </ToolTip>
            </div>
          ))}
        </div>

        <ToolTip content={isExpanded ? "Collapse" : "Expand"} position="bottom" delay={400}>
          <button
            onClick={() => setIsExpanded(!isExpanded)}
            className={`p-2 rounded hover:bg-[var(--color-surface-hover)] cursor-pointer text-[var(--color-text-secondary)] hover:text-[var(--color-text)] transition-all ease-in-out duration-300 ${
              isExpanded ? 'ml-2' : 'ml-0'
            }`}
          >
            {isExpanded ? (
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <polyline points="15,18 9,12 15,6" />
              </svg>
            ) : (
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <polyline points="9,18 15,12 9,6" />
              </svg>
            )}
          </button>
        </ToolTip>
      </div>
    </div>
  );
};

const TerminalExample = () => {
  const [lines] = useState([
    { id: '1', content: 'Terminal instance for: Example Server', type: 'system', timestamp: new Date() },
    { id: '2', content: '> help', type: 'command', timestamp: new Date() },
    { id: '3', content: 'Available commands: /help, /list, /tp, /gamemode', type: 'output', timestamp: new Date() },
    { id: '4', content: '> list', type: 'command', timestamp: new Date() },
    { id: '5', content: 'There are 0 of a max of 20 players online:', type: 'output', timestamp: new Date() },
    { id: '6', content: '> gamemode creative @a', type: 'command', timestamp: new Date() },
    { id: '7', content: 'Set own game mode to Creative Mode', type: 'output', timestamp: new Date() }
  ]);

  const getLineColor = (type: string) => {
    switch (type) {
      case 'command':
        return 'var(--color-primary)'; // Primary color for commands
      case 'error':
        return 'var(--color-danger)'; // Danger color for errors
      case 'system':
        return 'var(--color-warning)'; // Warning color for system messages
      default:
        return '#d1d5db'; // Light gray for better contrast on dark terminal background
    }
  };

  return (
    <div className="h-full flex flex-col bg-[#2B2B2B] border border-gray-900 rounded-lg overflow-hidden">
      {/* Terminal Header */}
      <div className="bg-[#2B2B2B] border-b border-gray-700 px-4 py-2 flex items-center justify-between flex-shrink-0">
        <div className="text-gray-300 text-sm font-medium">
          Terminal - Example Server
        </div>
        <button className="p-1.5 rounded hover:bg-gray-500 transition-colors duration-200 text-gray-300 hover:text-white">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8" />
            <path d="M21 3v5h-5" />
            <path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16" />
            <path d="M3 21v-5h5" />
          </svg>
        </button>
      </div>

      {/* Terminal Content */}
      <div className="flex-1 overflow-auto p-4 font-mono text-sm space-y-1">
        {lines.map((line) => (
          <div
            key={line.id}
            className="leading-relaxed"
            style={{ color: getLineColor(line.type) }}
          >
            {line.content}
          </div>
        ))}
      </div>

      {/* Command Input */}
      <div className="bg-[#2B2B2B] p-3 flex-shrink-0">
        <div className="flex items-center">
          <span className="text-gray-400 mr-2 font-mono">{'>'}</span>
          <div className="flex-1 bg-transparent text-gray-300 font-mono text-sm outline-none border border-gray-600 rounded px-2 py-1">
            Enter Minecraft command...
          </div>
        </div>
      </div>
    </div>
  );
};