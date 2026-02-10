class MusicLibrary {
  constructor() {
      this.baseUrl = 'http://localhost:4000';
      this.currentTrack = null;
      this.confirmCallback = null;
      this.observers = new Map();
      this.init();
  }

  init() {
      this.initTheme();
      this.bindEvents();
      this.loadTracks();
      this.initIntersectionObserver();
  }

  initTheme() {
      const theme = localStorage.getItem('theme') || 
                   (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light');
      document.documentElement.setAttribute('data-theme', theme);
      document.getElementById('themeSwitch').checked = theme === 'dark';
  }

  bindEvents() {
      // Theme toggle
      document.getElementById('themeSwitch').addEventListener('change', (e) => {
          const theme = e.target.checked ? 'dark' : 'light';
          document.documentElement.setAttribute('data-theme', theme);
          localStorage.setItem('theme', theme);
      });

      // Add track buttons
      document.getElementById('addTrackBtn').addEventListener('click', () => this.showModal('addTrackModal'));
      document.getElementById('addFirstTrackBtn').addEventListener('click', () => this.showModal('addTrackModal'));

      // Form submissions
      document.getElementById('addTrackForm').addEventListener('submit', (e) => this.handleAddTrack(e));
      document.getElementById('editTrackForm').addEventListener('submit', (e) => this.handleEditTrack(e));

      // Modal close buttons
      document.querySelectorAll('.modal-close').forEach(btn => {
          btn.addEventListener('click', () => this.closeModals());
      });

      // Edit track button
      document.getElementById('editTrackBtn').addEventListener('click', () => this.showEditModal());

      // Confirmation modal
      document.getElementById('confirmCancel').addEventListener('click', () => this.closeModals());
      document.getElementById('confirmOk').addEventListener('click', () => {
          if (this.confirmCallback) this.confirmCallback();
          this.closeModals();
      });

      // Close modals on outside click
      document.querySelectorAll('.modal').forEach(modal => {
          modal.addEventListener('click', (e) => {
              if (e.target === modal) this.closeModals();
          });
      });
  }

  async loadTracks() {
      try {
          const response = await fetch(`${this.baseUrl}/trackls`);
          if (!response.ok) throw new Error('Failed to load tracks');
          
          const tracks = await response.json();
          this.renderTracks(tracks);
          
          if (tracks.length === 0) {
              document.getElementById('emptyState').classList.remove('hidden');
              document.getElementById('trackList').innerHTML = '';
          } else {
              document.getElementById('emptyState').classList.add('hidden');
          }
      } catch (error) {
          this.showToast('Error loading tracks', 'error');
          console.error('Error loading tracks:', error);
      } finally {
          document.getElementById('loading').classList.add('hidden');
      }
  }

  renderTracks(tracks) {
      const trackList = document.getElementById('trackList');
      trackList.innerHTML = '';

      tracks.forEach(track => {
          const card = this.createTrackCard(track);
          trackList.appendChild(card);
          this.observeTrack(card, track);
      });
  }

  createTrackCard(track) {
      const card = document.createElement('div');
      card.className = 'track-card';
      card.dataset.provider = track.provider;
      card.dataset.id = track.id;
      
      card.innerHTML = `
          <div class="track-info">
              <div class="track-title">${track.metadata?.title || 'Loading...'}</div>
              <div class="track-meta">
                  <span><i class="fas fa-user"></i> ${track.metadata?.artists?.join(', ') || 'Unknown Artist'}</span>
                  <span><i class="fas fa-compact-disc"></i> ${track.metadata?.album || 'Unknown Album'}</span>
              </div>
          </div>
          <div class="track-actions">
              <button class="btn-autotag" title="Auto-tag" data-provider="${track.provider}" data-id="${track.id}">
                  <i class="fas fa-robot"></i>
              </button>
              <button class="btn-delete" title="Delete" data-provider="${track.provider}" data-id="${track.id}">
                  <i class="fas fa-trash"></i>
              </button>
          </div>
      `;

      // Add click handlers
      card.addEventListener('click', (e) => {
          if (!e.target.closest('.track-actions')) {
              this.showTrackDetails(track.provider, track.id);
          }
      });

      const autotagBtn = card.querySelector('.btn-autotag');
      autotagBtn.addEventListener('click', (e) => {
          e.stopPropagation();
          this.showAutotagModal(track.provider, track.id);
      });

      const deleteBtn = card.querySelector('.btn-delete');
      deleteBtn.addEventListener('click', (e) => {
          e.stopPropagation();
          this.confirmDelete(track.provider, track.id);
      });

      return card;
  }

  initIntersectionObserver() {
      this.intersectionObserver = new IntersectionObserver((entries) => {
          entries.forEach(entry => {
              if (entry.isIntersecting) {
                  const card = entry.target;
                  const provider = card.dataset.provider;
                  const id = card.dataset.id;
                  this.loadTrackMetadata(card, provider, id);
                  this.intersectionObserver.unobserve(card);
              }
          });
      }, {
          rootMargin: '50px',
          threshold: 0.1
      });
  }

  observeTrack(card, track) {
      // Only observe if we don't have metadata yet
      if (!track.metadata || !track.metadata.title) {
          this.intersectionObserver.observe(card);
      }
  }

  async loadTrackMetadata(card, provider, id) {
      try {
          const response = await fetch(`${this.baseUrl}/track/${provider}/${id}`);
          if (!response.ok) return;
          
          const metadata = await response.json();
          this.updateTrackCard(card, metadata);
      } catch (error) {
          console.error('Error loading track metadata:', error);
      }
  }

  updateTrackCard(card, metadata) {
      const titleEl = card.querySelector('.track-title');
      const metaEl = card.querySelector('.track-meta');
      
      titleEl.textContent = metadata.title || 'Unknown Title';
      metaEl.innerHTML = `
          <span><i class="fas fa-user"></i> ${metadata.artists?.join(', ') || 'Unknown Artist'}</span>
          <span><i class="fas fa-compact-disc"></i> ${metadata.album || 'Unknown Album'}</span>
      `;
  }

  async showTrackDetails(provider, id) {
      this.showModal('trackDetailsModal');
      
      const loadingEl = document.getElementById('trackDetailsLoading');
      const contentEl = document.getElementById('trackDetailsContent');
      
      loadingEl.classList.remove('hidden');
      contentEl.classList.add('hidden');
      
      try {
          const response = await fetch(`${this.baseUrl}/track/${provider}/${id}`);
          if (!response.ok) throw new Error('Failed to load track details');
          
          const metadata = await response.json();
          this.displayTrackDetails(metadata);
          
          // Store current track for editing
          this.currentTrack = { provider, id, metadata };
          
          loadingEl.classList.add('hidden');
          contentEl.classList.remove('hidden');
      } catch (error) {
          this.showToast('Error loading track details', 'error');
          console.error('Error loading track details:', error);
      }
  }

  displayTrackDetails(metadata) {
      const formatList = (list) => list?.length > 0 ? list.join(', ') : 'None';
      
      document.getElementById('detailTitle').textContent = metadata.title || 'Unknown';
      document.getElementById('detailArtists').textContent = formatList(metadata.artists);
      document.getElementById('detailAlbum').textContent = metadata.album || 'Unknown';
      document.getElementById('detailDate').textContent = metadata.date || 'Unknown';
      document.getElementById('detailGenres').textContent = formatList(metadata.genres);
      document.getElementById('detailIsrc').textContent = metadata.isrc || 'Unknown';
      document.getElementById('detailLyrics').textContent = metadata.lyrics || 'No lyrics available';
  }

  showEditModal() {
      if (!this.currentTrack) return;
      
      const { metadata } = this.currentTrack;
      
      document.getElementById('editProvider').value = this.currentTrack.provider;
      document.getElementById('editId').value = this.currentTrack.id;
      document.getElementById('editTitle').value = metadata.title || '';
      document.getElementById('editArtists').value = metadata.artists?.join('\n') || '';
      document.getElementById('editAlbum').value = metadata.album || '';
      document.getElementById('editDate').value = metadata.date || '';
      document.getElementById('editGenres').value = metadata.genres?.join(', ') || '';
      document.getElementById('editLyrics').value = metadata.lyrics || '';
      document.getElementById('editIsrc').value = metadata.isrc || '';
      
      this.closeModals();
      this.showModal('editTrackModal');
  }

  async handleAddTrack(e) {
      e.preventDefault();
      
      const form = e.target;
      const provider = form.provider.value;
      const id = this.extractVideoId(form.trackId.value);
      const bulkJson = form.bulkTracks.value.trim();
      
      let tracks = [];
      
      if (bulkJson) {
          try {
              tracks = JSON.parse(bulkJson);
              if (!Array.isArray(tracks)) {
                  throw new Error('JSON must be an array');
              }
          } catch (error) {
              this.showToast('Invalid JSON format', 'error');
              return;
          }
      } else if (id) {
          tracks = [{ provider, id }];
      } else {
          this.showToast('Please enter a track ID or JSON', 'error');
          return;
      }
      
      try {
          const response = await fetch(`${this.baseUrl}/trackadd`, {
              method: 'POST',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify(tracks)
          });
          
          if (!response.ok) throw new Error('Failed to add tracks');
          
          this.showToast('Tracks added successfully', 'success');
          this.closeModals();
          this.loadTracks();
          form.reset();
      } catch (error) {
          this.showToast('Error adding tracks', 'error');
          console.error('Error adding tracks:', error);
      }
  }

  async handleEditTrack(e) {
      e.preventDefault();
      
      const form = e.target;
      const provider = form.provider.value;
      const id = form.id.value;
      
      const metadata = {
          title: form.title.value || null,
          artists: form.artists.value.split('\n').filter(a => a.trim()),
          album: form.album.value || null,
          date: form.date.value || null,
          genres: form.genres.value.split(',').map(g => g.trim()).filter(g => g),
          lyrics: form.lyrics.value || null,
          isrc: form.isrc.value || null
      };
      
      try {
          const response = await fetch(`${this.baseUrl}/track/${provider}/${id}`, {
              method: 'PATCH',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify(metadata)
          });
          
          if (!response.ok) throw new Error('Failed to update track');
          
          this.showToast('Track updated successfully', 'success');
          this.closeModals();
          this.loadTracks();
      } catch (error) {
          this.showToast('Error updating track', 'error');
          console.error('Error updating track:', error);
      }
  }

  async showAutotagModal(provider, id) {
      this.currentTrack = { provider, id };
      this.showModal('autotagModal');
      
      const loadingEl = document.getElementById('autotagLoading');
      const contentEl = document.getElementById('autotagContent');
      
      loadingEl.classList.remove('hidden');
      contentEl.classList.add('hidden');
      
      try {
          const response = await fetch(`${this.baseUrl}/track/${provider}/${id}/autotag`);
          if (!response.ok) throw new Error('Failed to load autotag suggestions');
          
          const candidates = await response.json();
          this.displayAutotagCandidates(candidates);
          
          loadingEl.classList.add('hidden');
          contentEl.classList.remove('hidden');
      } catch (error) {
          this.showToast('Error loading autotag suggestions', 'error');
          console.error('Error loading autotag suggestions:', error);
      }
  }

  displayAutotagCandidates(candidates) {
      const container = document.getElementById('autotagCandidates');
      container.innerHTML = '';
      
      if (candidates.length === 0) {
          container.innerHTML = '<p class="no-candidates">No suggestions found</p>';
          return;
      }
      
      candidates.forEach((candidate, index) => {
          const card = document.createElement('div');
          card.className = 'candidate-card';
          card.innerHTML = `
              <div class="candidate-title">${candidate.title || 'Untitled'}</div>
              <div class="candidate-meta">
                  <span>Artists: ${candidate.artists?.join(', ') || 'Unknown'}</span>
                  <span>Album: ${candidate.album || 'Unknown'}</span>
                  <span>Date: ${candidate.date || 'Unknown'}</span>
              </div>
          `;
          
          card.addEventListener('click', () => this.selectAutotagCandidate(candidate));
          container.appendChild(card);
      });
  }

  async selectAutotagCandidate(candidate) {
      if (!this.currentTrack) return;
      
      const { provider, id } = this.currentTrack;
      
      try {
          // First update the track with selected metadata
          const response = await fetch(`${this.baseUrl}/track/${provider}/${id}`, {
              method: 'PATCH',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify(candidate)
          });
          
          if (!response.ok) throw new Error('Failed to apply autotag');
          
          this.showToast('Autotag applied successfully', 'success');
          this.closeModals();
          this.loadTracks();
      } catch (error) {
          this.showToast('Error applying autotag', 'error');
          console.error('Error applying autotag:', error);
      }
  }

  confirmDelete(provider, id) {
      this.currentTrack = { provider, id };
      this.confirmCallback = () => this.deleteTrack(provider, id);
      
      document.getElementById('confirmMessage').textContent = 
          `Are you sure you want to delete this track? (${provider}:${id})`;
      
      this.showModal('confirmModal');
  }

  async deleteTrack(provider, id) {
      try {
          const response = await fetch(`${this.baseUrl}/track/${provider}/${id}`, {
              method: 'DELETE'
          });
          
          if (!response.ok) throw new Error('Failed to delete track');
          
          this.showToast('Track deleted successfully', 'success');
          this.loadTracks();
      } catch (error) {
          this.showToast('Error deleting track', 'error');
          console.error('Error deleting track:', error);
      }
  }

  showModal(modalId) {
      this.closeModals();
      document.getElementById(modalId).classList.remove('hidden');
      document.body.style.overflow = 'hidden';
  }

  closeModals() {
      document.querySelectorAll('.modal').forEach(modal => {
          modal.classList.add('hidden');
      });
      document.body.style.overflow = '';
      this.confirmCallback = null;
  }

  extractVideoId(urlOrId) {
      if (!urlOrId) return '';
      
      // If it's already an ID (no special characters except maybe dashes and underscores)
      if (/^[a-zA-Z0-9_-]{11}$/.test(urlOrId)) {
          return urlOrId;
      }
      
      // Try to extract from YouTube URL
      const patterns = [
          /(?:youtube\.com\/watch\?v=|youtu\.be\/)([a-zA-Z0-9_-]{11})/,
          /(?:embed\/)([a-zA-Z0-9_-]{11})/,
          /v=([a-zA-Z0-9_-]{11})/
      ];
      
      for (const pattern of patterns) {
          const match = urlOrId.match(pattern);
          if (match) return match[1];
      }
      
      return urlOrId;
  }

  showToast(message, type = 'info') {
      const container = document.getElementById('toastContainer');
      const toast = document.createElement('div');
      toast.className = `toast ${type}`;
      toast.innerHTML = `
          <div class="toast-message">${message}</div>
          <button class="toast-close">&times;</button>
      `;
      
      container.appendChild(toast);
      
      // Auto-remove after 5 seconds
      setTimeout(() => {
          toast.style.opacity = '0';
          setTimeout(() => toast.remove(), 300);
      }, 5000);
      
      // Close button
      toast.querySelector('.toast-close').addEventListener('click', () => {
          toast.style.opacity = '0';
          setTimeout(() => toast.remove(), 300);
      });
  }
}

// Initialize the application
document.addEventListener('DOMContentLoaded', () => {
  new MusicLibrary();
});