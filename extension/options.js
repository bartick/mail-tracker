/**
 * @fileoverview Handles saving and restoring the extension's options.
 */

// Saves options to chrome.storage.sync.
function saveOptions() {
  const trackerUrl = document.getElementById('trackerUrl').value;
  chrome.storage.sync.set({
    trackerBaseUrl: trackerUrl
  }, () => {
    // Update status to let user know options were saved.
    const status = document.getElementById('status');
    status.textContent = 'Options saved.';
    setTimeout(() => {
      status.textContent = '';
    }, 1500);
  });
}

// Restores the input field state using preferences stored in chrome.storage.
function restoreOptions() {
  chrome.storage.sync.get({
    trackerBaseUrl: '' // Default to an empty string
  }, (items) => {
    document.getElementById('trackerUrl').value = items.trackerBaseUrl;
  });
}

// Event listeners for DOM content loaded and save button click.
document.addEventListener('DOMContentLoaded', restoreOptions);
document.getElementById('save').addEventListener('click', saveOptions);
