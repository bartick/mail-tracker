/**
 * @fileoverview This content script finds Gmail compose windows and injects
 * a dynamic tracking pixel. The tracker URL is loaded from extension storage.
 * This version includes enhanced logging for easier debugging.
 */

console.log("Gmail HTML Injector v1.3: Script loaded and running.");

// --- Configuration ---
// This will be populated from chrome.storage.sync
let TRACKER_BASE_URL = ""; 
const SIGNATURE_ID = 'gmail-signature-tracker-block';
const TRACKER_IMG_ID = 'gmail-tracker-pixel-image';

/**
 * Creates the full HTML for the signature and tracking pixel.
 * @param {string} trackingUrl The fully formed URL for the tracking pixel.
 * @returns {string} The HTML string to be injected.
 */
function createSignatureHtml(trackingUrl) {
  return `
    <div id="${SIGNATURE_ID}" style="padding-top: 15px; margin-top: 20px;">
      <p style="margin: 0; padding: 0; font-family: Arial, sans-serif; font-size: 12.8px; color: #333;">
        Best regards,<br>
        Your Name
      </p>
      <img id="${TRACKER_IMG_ID}" src="${trackingUrl}" width="1" height="1" alt="" style="border:0;width:1px;height:1px;">
    </div>
  `;
}

/**
 * Injects or updates the tracking pixel in a given compose window.
 * It dynamically builds the pixel URL based on recipient and subject fields.
 * @param {HTMLElement} composeEditor - The content-editable div of the compose window.
 */
function injectOrUpdateTracker(composeEditor) {
  if (!TRACKER_BASE_URL) {
    // This check is a safeguard, but the main check is done at startup.
    return;
  }

  const composeContainer = composeEditor.closest('div[role="dialog"]');
  if (!composeContainer) {
    return;
  }

  const recipientElements = composeContainer.querySelectorAll('span[email]');
  const recipients = Array.from(recipientElements).map(el => el.getAttribute('email')).join(',');

  const subjectElement = composeContainer.querySelector('input[name="subjectbox"]');
  const subject = subjectElement ? subjectElement.value : '';

  const params = new URLSearchParams();
  if (recipients) params.append('email', recipients);
  if (subject) params.append('subject', subject);
  
  const trackingUrl = `${TRACKER_BASE_URL}?${params.toString()}`;

  const existingSignature = composeEditor.querySelector(`#${SIGNATURE_ID}`);
  
  if (existingSignature) {
    const trackerImage = existingSignature.querySelector(`#${TRACKER_IMG_ID}`);
    if (trackerImage && trackerImage.src !== trackingUrl) {
      trackerImage.src = trackingUrl;
      console.log('Injector: Tracker URL updated.');
    }
  } else {
    const isReplyOrForward = composeEditor.querySelector('blockquote.gmail_quote');
    if (!isReplyOrForward) {
      console.log('Injector: New email detected. Injecting signature with URL:', trackingUrl);
      const signatureHtml = createSignatureHtml(trackingUrl);
      composeEditor.insertAdjacentHTML('beforeend', signatureHtml);
    }
  }
}

/**
 * Scans the document for visible Gmail compose windows.
 */
function findAndProcessComposeWindows() {
  const composeEditors = document.querySelectorAll('div[aria-label="Message Body"]');
  if (composeEditors.length > 0) {
    composeEditors.forEach(injectOrUpdateTracker);
  }
}


// --- Main Execution ---

// Load the tracker URL from storage first. The rest of the script will not run
// until we have this value.
chrome.storage.sync.get('trackerBaseUrl', (data) => {
  if (data.trackerBaseUrl && data.trackerBaseUrl.trim() !== '') {
    TRACKER_BASE_URL = data.trackerBaseUrl;
    console.log("Injector SUCCESS: Loaded tracker URL from storage:", TRACKER_BASE_URL);

    // Now that the URL is loaded, run the main logic.
    findAndProcessComposeWindows();

    const observer = new MutationObserver(() => {
      findAndProcessComposeWindows();
    });

    observer.observe(document.body, {
      childList: true,
      subtree: true,
    });
  } else {
    // This is the most likely reason for failure.
    console.error("Injector STOP: No tracker URL found. Please right-click the extension icon, go to 'Options', and save your tracker URL.");
  }
});

