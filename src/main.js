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

function renderContainerRow(c) {
  const statusClass = getStatusClass(c.status);
  const running = isRunning(c.status);
  return '<tr>' +
    '<td title="' + escapeHtml(c.names) + '">' + escapeHtml(c.names) + '</td>' +
    '<td title="' + escapeHtml(c.image) + '">' + escapeHtml(c.image) + '</td>' +
    '<td><span class="status-badge"><span class="status-dot ' + statusClass + '"></span>' + escapeHtml(c.status) + '</span></td>' +
    '<td title="' + escapeHtml(c.ports) + '">' + (c.ports ? escapeHtml(c.ports) : '\u2014') + '</td>' +
    '<td><span class="container-id">' + escapeHtml(c.id.substring(0, 12)) + '</span></td>' +
    '<td>' +
      (running
        ? '<button class="action-btn stop" onclick="containerAction(\'' + c.id + '\', \'stop\')">Stop</button>' +
          '<button class="action-btn" onclick="containerAction(\'' + c.id + '\', \'restart\')">Restart</button>'
        : '<button class="action-btn start" onclick="containerAction(\'' + c.id + '\', \'start\')">Start</button>' +
          '<button class="action-btn remove" onclick="containerAction(\'' + c.id + '\', \'rm\')">Remove</button>') +
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
        ? '<button class="action-btn stop group-action-btn" onclick="event.stopPropagation(); composeAction(\'' + escapeHtml(project) + '\', \'stop\')">Stop All</button>' +
          '<button class="action-btn group-action-btn" onclick="event.stopPropagation(); composeAction(\'' + escapeHtml(project) + '\', \'restart\')">Restart All</button>'
        : (runCount === 0
          ? '<button class="action-btn start group-action-btn" onclick="event.stopPropagation(); composeAction(\'' + escapeHtml(project) + '\', \'start\')">Start All</button>'
          : '<button class="action-btn start group-action-btn" onclick="event.stopPropagation(); composeAction(\'' + escapeHtml(project) + '\', \'start\')">Start All</button>' +
            '<button class="action-btn stop group-action-btn" onclick="event.stopPropagation(); composeAction(\'' + escapeHtml(project) + '\', \'stop\')">Stop All</button>');

      html += '<tr class="group-header" onclick="toggleGroup(\'' + escapeHtml(project) + '\')">' +
        '<td colspan="6">' +
          '<span class="group-chevron">' + chevron + '</span>' +
          '<span class="group-dot ' + groupStatus + '"></span>' +
          '<span class="group-name">' + escapeHtml(project) + '</span>' +
          '<span class="group-count">' + runCount + '/' + totalCount + ' running</span>' +
          '<span class="group-actions">' + groupButtons + '</span>' +
        '</td>' +
      '</tr>';

      if (!collapsed) {
        containers.forEach(function(c) { html += renderContainerRow(c); });
      }
    } else {
      containers.forEach(function(c) { html += renderContainerRow(c); });
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
  } catch (e) {
    loading.style.display = 'none';
    error.style.display = 'flex';
    error.textContent = 'Error: ' + e;
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

window.addEventListener('DOMContentLoaded', function() {
  document.getElementById('refresh-btn').addEventListener('click', loadContainers);

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
