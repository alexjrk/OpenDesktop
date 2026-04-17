const { invoke } = window.__TAURI__.core;

let allContainers = [];
let currentFilter = 'all';
let collapsedGroups = new Set();

function getStatusClass(status) {
  const s = status.toLowerCase();
  if (s.startsWith('up')) return 'running';
  if (s.includes('exited')) return 'exited';
  if (s.includes('paused')) return 'paused';
  if (s.includes('created')) return 'created';
  if (s.includes('restarting')) return 'restarting';
  return 'exited';
}

function isRunning(status) {
  return status.toLowerCase().startsWith('up');
}

function filterContainers(containers, filter) {
  if (filter === 'all') return containers;
  if (filter === 'running') return containers.filter(c => isRunning(c.status));
  if (filter === 'stopped') return containers.filter(c => !isRunning(c.status));
  return containers;
}

function showToast(message, type) {
  const existing = document.querySelector('.toast');
  if (existing) existing.remove();

  const toast = document.createElement('div');
  toast.className = 'toast ' + type + '-toast';
  toast.textContent = message;
  document.body.appendChild(toast);
  setTimeout(function() { toast.remove(); }, 3000);
}

function groupByProject(containers) {
  const groups = new Map();
  containers.forEach(function(c) {
    const key = c.project || '';
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key).push(c);
  });
  return groups;
}

function escapeHtml(str) {
  return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

function renderContainerRow(c, isCompose) {
  const statusClass = getStatusClass(c.status);
  const running = isRunning(c.status);
  const rowClass = isCompose ? ' class="compose-child"' : '';
  return '<tr' + rowClass + ' data-container-id="' + escapeHtml(c.id) + '" data-container-name="' + escapeHtml(c.names) + '">' +
    '<td title="' + escapeHtml(c.names) + '">' + escapeHtml(c.names) + '</td>' +
    '<td title="' + escapeHtml(c.image) + '">' + escapeHtml(c.image) + '</td>' +
    '<td><span class="status-badge"><span class="status-dot ' + statusClass + '"></span>' + escapeHtml(c.status) + '</span></td>' +
    '<td title="' + escapeHtml(c.ports) + '">' + (c.ports ? escapeHtml(c.ports) : '\u2014') + '</td>' +
    '<td><span class="container-id">' + escapeHtml(c.id.substring(0, 12)) + '</span></td>' +
    '<td>' +
      (running
        ? '<button class="action-btn stop" data-action="stop" data-id="' + escapeHtml(c.id) + '">Stop</button>' +
          '<button class="action-btn" data-action="restart" data-id="' + escapeHtml(c.id) + '">Restart</button>'
        : '<button class="action-btn start" data-action="start" data-id="' + escapeHtml(c.id) + '">Start</button>' +
          '<button class="action-btn remove" data-action="rm" data-id="' + escapeHtml(c.id) + '">Remove</button>') +
    '</td>' +
  '</tr>';
}

function renderContainers(containers) {
  const tbody = document.getElementById('containers-body');
  const loading = document.getElementById('loading');
  const empty = document.getElementById('empty');
  const error = document.getElementById('error');
  const tableWrapper = document.querySelector('.table-wrapper');
  const countEl = document.getElementById('container-count');

  loading.style.display = 'none';
  error.style.display = 'none';

  const filtered = filterContainers(containers, currentFilter);
  const running = containers.filter(function(c) { return isRunning(c.status); }).length;
  countEl.textContent = running + ' running / ' + containers.length + ' total';

  if (filtered.length === 0) {
    tableWrapper.style.display = 'none';
    empty.style.display = 'flex';
    return;
  }

  tableWrapper.style.display = 'block';
  empty.style.display = 'none';

  const groups = groupByProject(filtered);
  let html = '';

  groups.forEach(function(containers, project) {
    if (project) {
      const runCount = containers.filter(function(c) { return isRunning(c.status); }).length;
      const totalCount = containers.length;
      const allUp = runCount === totalCount;
      const collapsed = collapsedGroups.has(project);
      const chevron = collapsed ? '\u25b6' : '\u25bc';
      const groupStatus = allUp ? 'running' : (runCount === 0 ? 'exited' : 'partial');

      const groupButtons = allUp
        ? '<button class="action-btn stop group-action-btn" data-compose-action="stop" data-project="' + escapeHtml(project) + '">Stop All</button>' +
          '<button class="action-btn group-action-btn" data-compose-action="restart" data-project="' + escapeHtml(project) + '">Restart All</button>'
        : (runCount === 0
          ? '<button class="action-btn start group-action-btn" data-compose-action="start" data-project="' + escapeHtml(project) + '">Start All</button>' +
            '<button class="action-btn remove group-action-btn" data-compose-action="down" data-project="' + escapeHtml(project) + '">Remove All</button>'
          : '<button class="action-btn start group-action-btn" data-compose-action="start" data-project="' + escapeHtml(project) + '">Start All</button>' +
            '<button class="action-btn stop group-action-btn" data-compose-action="stop" data-project="' + escapeHtml(project) + '">Stop All</button>');

      html += '<tr class="group-header" data-project="' + escapeHtml(project) + '">' +
        '<td colspan="6">' +
          '<span class="group-chevron">' + chevron + '</span>' +
          '<span class="group-dot ' + groupStatus + '"></span>' +
          '<span class="group-name">' + escapeHtml(project) + '</span>' +
          '<span class="group-count">' + runCount + '/' + totalCount + ' running</span>' +
          '<span class="group-actions">' + groupButtons + '</span>' +
        '</td>' +
      '</tr>';

      if (!collapsed) {
        containers.forEach(function(c) { html += renderContainerRow(c, true); });
      }
    } else {
      containers.forEach(function(c) { html += renderContainerRow(c, false); });
    }
  });

  tbody.innerHTML = html;
}

window.toggleGroup = function(project) {
  if (collapsedGroups.has(project)) {
    collapsedGroups.delete(project);
  } else {
    collapsedGroups.add(project);
  }
  renderContainers(allContainers);
};

window.composeAction = async function(project, action) {
  try {
    await invoke('compose_action', { project, action });
    showToast('Compose ' + project + ' ' + action + ' successful', 'success');
    setTimeout(loadContainers, 1000);
  } catch (e) {
    showToast('Failed: ' + e, 'error');
  }
};

function hideLoadingOverlay() {
  const overlay = document.getElementById('loading-overlay');
  if (overlay) {
    overlay.classList.add('fade-out');
    setTimeout(function() { overlay.remove(); }, 300);
  }
}

async function loadContainers() {
  const loading = document.getElementById('loading');
  const error = document.getElementById('error');
  const tableWrapper = document.querySelector('.table-wrapper');
  const empty = document.getElementById('empty');

  loading.style.display = 'flex';
  error.style.display = 'none';
  tableWrapper.style.display = 'none';
  empty.style.display = 'none';

  try {
    allContainers = await invoke('get_containers');
    renderContainers(allContainers);
    hideLoadingOverlay();
  } catch (e) {
    loading.style.display = 'none';
    error.style.display = 'flex';
    error.textContent = 'Error: ' + e;
    hideLoadingOverlay();
  }
}

window.containerAction = async function(id, action) {
  try {
    await invoke('container_action', { id, action });
    showToast('Container ' + action + ' successful', 'success');
    setTimeout(loadContainers, 500);
  } catch (e) {
    showToast('Failed: ' + e, 'error');
  }
};

let currentLogsId = null;
let currentLogsName = null;
let currentLogsText = '';
let searchMatches = [];
let currentMatchIndex = -1;
let logsRequestToken = 0;

async function fetchLogs(id, name) {
  const token = ++logsRequestToken;
  const content = document.getElementById('logs-content');
  content.innerHTML = '<span class="logs-loading">Loading logs...</span>';
  try {
    const logs = await invoke('get_container_logs', { id, tail: 200 });
    if (token !== logsRequestToken) return;
    currentLogsText = logs || '(no logs)';
    content.textContent = currentLogsText;
    content.scrollTop = content.scrollHeight;
    // Re-apply search if search bar is visible
    const searchBar = document.getElementById('logs-search');
    const searchInput = document.getElementById('logs-search-input');
    if (searchBar.style.display !== 'none' && searchInput.value) {
      applyLogSearch(searchInput.value);
    }
  } catch (e) {
    if (token !== logsRequestToken) return;
    currentLogsText = '';
    content.textContent = 'Error fetching logs: ' + e;
  }
}

function escapeRegExp(str) {
  return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function applyLogSearch(query) {
  const content = document.getElementById('logs-content');
  const countEl = document.getElementById('logs-search-count');
  searchMatches = [];
  currentMatchIndex = -1;

  if (!query || !currentLogsText) {
    content.textContent = currentLogsText;
    countEl.textContent = '';
    return;
  }

  const regex = new RegExp(escapeRegExp(query), 'gi');
  let match;
  let lastIndex = 0;
  let html = '';
  let i = 0;

  while ((match = regex.exec(currentLogsText)) !== null) {
    html += escapeHtml(currentLogsText.substring(lastIndex, match.index));
    html += '<mark data-match="' + i + '">' + escapeHtml(match[0]) + '</mark>';
    searchMatches.push(i);
    lastIndex = regex.lastIndex;
    i++;
  }
  html += escapeHtml(currentLogsText.substring(lastIndex));

  content.innerHTML = html;

  if (searchMatches.length > 0) {
    currentMatchIndex = 0;
    highlightCurrentMatch();
    countEl.textContent = '1 / ' + searchMatches.length;
  } else {
    countEl.textContent = 'No results';
  }
}

function highlightCurrentMatch() {
  const content = document.getElementById('logs-content');
  content.querySelectorAll('mark.current').forEach(function(m) { m.classList.remove('current'); });
  if (currentMatchIndex >= 0 && currentMatchIndex < searchMatches.length) {
    const mark = content.querySelector('mark[data-match="' + currentMatchIndex + '"]');
    if (mark) {
      mark.classList.add('current');
      mark.scrollIntoView({ block: 'center', behavior: 'smooth' });
    }
  }
}

function navigateMatch(dir) {
  if (searchMatches.length === 0) return;
  currentMatchIndex = (currentMatchIndex + dir + searchMatches.length) % searchMatches.length;
  highlightCurrentMatch();
  document.getElementById('logs-search-count').textContent = (currentMatchIndex + 1) + ' / ' + searchMatches.length;
}

function openLogSearch() {
  const panel = document.getElementById('logs-panel');
  if (panel.style.display === 'none') return;
  const searchBar = document.getElementById('logs-search');
  const input = document.getElementById('logs-search-input');
  searchBar.style.display = 'flex';
  input.focus();
  input.select();
}

function closeLogSearch() {
  var searchBar = document.getElementById('logs-search');
  var input = document.getElementById('logs-search-input');
  var content = document.getElementById('logs-content');
  var countEl = document.getElementById('logs-search-count');
  searchBar.style.display = 'none';
  input.value = '';
  countEl.textContent = '';
  searchMatches = [];
  currentMatchIndex = -1;
  content.textContent = currentLogsText;
}

window.openLogs = async function(id, name) {
  const panel = document.getElementById('logs-panel');
  const title = document.getElementById('logs-title');
  const app = document.querySelector('.app');

  currentLogsId = id;
  currentLogsName = name;

  // Highlight active row
  document.querySelectorAll('tr.active-row').forEach(function(r) { r.classList.remove('active-row'); });
  const row = document.querySelector('tr[data-container-id="' + CSS.escape(id) + '"]');
  if (row) row.classList.add('active-row');

  title.textContent = 'Logs — ' + name;
  panel.style.display = 'flex';
  app.classList.add('logs-open');

  await fetchLogs(id, name);
};

function closeLogs() {
  const panel = document.getElementById('logs-panel');
  const app = document.querySelector('.app');
  panel.style.display = 'none';
  app.classList.remove('logs-open');
  currentLogsId = null;
  currentLogsName = null;
  document.querySelectorAll('tr.active-row').forEach(function(r) { r.classList.remove('active-row'); });
}

window.addEventListener('DOMContentLoaded', function() {
  document.getElementById('refresh-btn').addEventListener('click', loadContainers);
  document.getElementById('logs-close-btn').addEventListener('click', closeLogs);
  document.getElementById('logs-refresh-btn').addEventListener('click', function() {
    if (currentLogsId) fetchLogs(currentLogsId, currentLogsName);
  });

  document.getElementById('logs-search-input').addEventListener('input', function(e) {
    applyLogSearch(e.target.value);
  });
  document.getElementById('logs-search-input').addEventListener('keydown', function(e) {
    if (e.key === 'Enter') {
      e.preventDefault();
      navigateMatch(e.shiftKey ? -1 : 1);
    }
    if (e.key === 'Escape') {
      e.preventDefault();
      closeLogSearch();
    }
  });
  document.getElementById('logs-search-prev').addEventListener('click', function() { navigateMatch(-1); });
  document.getElementById('logs-search-next').addEventListener('click', function() { navigateMatch(1); });
  document.getElementById('logs-search-close').addEventListener('click', closeLogSearch);

  document.addEventListener('keydown', function(e) {
    if ((e.ctrlKey || e.metaKey) && e.key === 'f') {
      var panel = document.getElementById('logs-panel');
      if (panel.style.display !== 'none') {
        e.preventDefault();
        openLogSearch();
      }
    }
  });

  document.getElementById('containers-body').addEventListener('click', function(e) {
    var composeBtn = e.target.closest('[data-compose-action]');
    if (composeBtn) {
      e.stopPropagation();
      composeAction(composeBtn.dataset.project, composeBtn.dataset.composeAction);
      return;
    }

    var actionBtn = e.target.closest('[data-action]');
    if (actionBtn) {
      e.stopPropagation();
      containerAction(actionBtn.dataset.id, actionBtn.dataset.action);
      return;
    }

    var groupHeader = e.target.closest('.group-header[data-project]');
    if (groupHeader) {
      toggleGroup(groupHeader.dataset.project);
      return;
    }

    var row = e.target.closest('tr[data-container-id][data-container-name]');
    if (row) {
      openLogs(row.dataset.containerId, row.dataset.containerName);
    }
  });

  document.querySelectorAll('.filter-btn').forEach(function(btn) {
    btn.addEventListener('click', function() {
      document.querySelectorAll('.filter-btn').forEach(function(b) { b.classList.remove('active'); });
      btn.classList.add('active');
      currentFilter = btn.dataset.filter;
      renderContainers(allContainers);
    });
  });

  loadContainers();
});
