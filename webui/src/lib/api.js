import { exec } from 'kernelsu';
import { DEFAULT_CONFIG, PATHS } from './constants';

function parseKvConfig(text) {
  try {
    const result = { ...DEFAULT_CONFIG };
    text.split('\n').forEach(line => {
      line = line.trim();
      if (!line || line.startsWith('#')) return;
      
      const eq = line.indexOf('=');
      if (eq < 0) return;
      
      let key = line.slice(0, eq).trim();
      let val = line.slice(eq + 1).trim();
      
      if ((val.startsWith('"') && val.endsWith('"')) || (val.startsWith("'") && val.endsWith("'"))) {
        val = val.slice(1, -1);
      }
      
      if (key === 'moduledir') result.moduledir = val;
      else if (key === 'tempdir') result.tempdir = val;
      else if (key === 'mountsource') result.mountsource = val;
      else if (key === 'verbose') result.verbose = (val === 'true');
      else if (key === 'partitions') result.partitions = val.split(',').map(s => s.trim()).filter(Boolean);
    });
    return result;
  } catch (e) { return null; }
}

function serializeKvConfig(cfg) {
  const q = s => `"${s}"`;
  const lines = ['# Hybrid Mount Config', ''];
  lines.push(`moduledir = ${q(cfg.moduledir)}`);
  if (cfg.tempdir) lines.push(`tempdir = ${q(cfg.tempdir)}`);
  lines.push(`mountsource = ${q(cfg.mountsource)}`);
  lines.push(`verbose = ${cfg.verbose}`);
  if (cfg.partitions.length) lines.push(`partitions = ${q(cfg.partitions.join(','))}`);
  return lines.join('\n');
}

export const API = {
  loadConfig: async () => {
    const { errno, stdout } = await exec(`[ -f "${PATHS.CONFIG}" ] && cat "${PATHS.CONFIG}" || echo ""`);
    if (errno !== 0) throw new Error('Failed to load config');
    return (stdout.trim()) ? (parseKvConfig(stdout) || DEFAULT_CONFIG) : DEFAULT_CONFIG;
  },

  saveConfig: async (config) => {
    const data = serializeKvConfig(config).replace(/'/g, "'\\''");
    const cmd = `mkdir -p "$(dirname "${PATHS.CONFIG}")" && printf '%s\n' '${data}' > "${PATHS.CONFIG}"`;
    const { errno } = await exec(cmd);
    if (errno !== 0) throw new Error('Failed to save config');
  },

  scanModules: async (moduleDir) => {
    const { stdout: modeOut } = await exec(`[ -f "${PATHS.MODE_CONFIG}" ] && cat "${PATHS.MODE_CONFIG}" || echo ""`);
    const modeMap = new Map();
    modeOut.split('\n').forEach(l => {
      const [id, m] = l.split('=').map(s => s.trim());
      if (id) modeMap.set(id, m);
    });

    const dir = moduleDir || DEFAULT_CONFIG.moduledir;
    const imgDir = PATHS.IMAGE_MNT;
    
    const cmd = `
      cd "${dir}" && for d in *;
      do
        if [ -d "$d" ] && [ ! -f "$d/disable" ] && [ ! -f "$d/skip_mount" ] && [ ! -f "$d/remove" ]; then
           HAS_CONTENT=false
           if [ -d "$d/system" ] || [ -d "$d/vendor" ] || [ -d "$d/product" ] || [ -d "$d/system_ext" ] || [ -d "$d/odm" ] || [ -d "$d/oem" ]; then
             HAS_CONTENT=true
           fi
           if [ "$HAS_CONTENT" = "false" ]; then
              if [ -d "${imgDir}/$d/system" ] || [ -d "${imgDir}/$d/vendor" ] || [ -d "${imgDir}/$d/product" ] || [ -d "${imgDir}/$d/system_ext" ] || [ -d "${imgDir}/$d/odm" ] || [ -d "${imgDir}/$d/oem" ]; then
                HAS_CONTENT=true
              fi
           fi
           if [ "$HAS_CONTENT" = "true" ]; then 
              NAME=$(grep "^name=" "$d/module.prop" 2>/dev/null | head -n1 | cut -d= -f2-)
              echo "$d|$NAME"
           fi
        fi
      done
    `;
    
    const { errno, stdout } = await exec(cmd);
    if (errno !== 0) throw new Error('Scan failed');

    return stdout.split('\n')
      .map(s => s.trim())
      .filter(s => s && !s.startsWith('meta-hybrid') && !s.startsWith('meta-overlayfs') && !s.startsWith('magic_mount'))
      .map(line => {
         const parts = line.split('|');
         const id = parts[0];
         const name = parts[1] || id; 
         return { id, name, mode: modeMap.get(id) || 'auto' };
      });
  },

  saveModules: async (modules) => {
    let content = "# Module Modes\n";
    modules.forEach(m => { if (m.mode !== 'auto') content += `${m.id}=${m.mode}\n`; });
    const data = content.replace(/'/g, "'\\''");
    const { errno } = await exec(`mkdir -p "$(dirname "${PATHS.MODE_CONFIG}")" && printf '%s\n' '${data}' > "${PATHS.MODE_CONFIG}"`);
    if (errno !== 0) throw new Error('Failed to save modes');
  },

  readLogs: async (logPath) => {
    const f = logPath || DEFAULT_CONFIG.logfile;
    const { errno, stdout, stderr } = await exec(`[ -f "${f}" ] && cat "${f}" || echo ""`);
    if (errno === 0 && stdout) return stdout;
    throw new Error(stdout || stderr || "Log file empty or not found");
  },

  fetchSystemColor: async () => {
    try {
      const { stdout } = await exec('settings get secure theme_customization_overlay_packages');
      if (stdout) {
        const match = /["']?android\.theme\.customization\.system_palette["']?\s*:\s*["']?#?([0-9a-fA-F]{6,8})["']?/i.exec(stdout) || 
                      /["']?source_color["']?\s*:\s*["']?#?([0-9a-fA-F]{6,8})["']?/i.exec(stdout);
        if (match && match[1]) {
          let hex = match[1];
          if (hex.length === 8) hex = hex.substring(2);
          return '#' + hex;
        }
      }
    } catch (e) {}
    return null;
  }
};