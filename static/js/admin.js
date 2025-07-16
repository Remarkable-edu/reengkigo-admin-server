/**
 * Admin System JavaScript
 * Handles frontend interactions for the admin interface
 */

// Initialize when DOM is ready
document.addEventListener('DOMContentLoaded', function() {
    initializeTooltips();
    initializeAlerts();
    initializeForms();
    initializeDataTables();
    initializeFileUploads();
});

/**
 * Initialize Bootstrap tooltips
 */
function initializeTooltips() {
    const tooltipTriggerList = [].slice.call(document.querySelectorAll('[data-bs-toggle="tooltip"]'));
    tooltipTriggerList.map(function (tooltipTriggerEl) {
        return new bootstrap.Tooltip(tooltipTriggerEl);
    });
}

/**
 * Initialize alert auto-dismiss
 */
function initializeAlerts() {
    const alerts = document.querySelectorAll('.alert[data-auto-dismiss]');
    alerts.forEach(alert => {
        const timeout = parseInt(alert.getAttribute('data-auto-dismiss')) || 5000;
        setTimeout(() => {
            const bsAlert = new bootstrap.Alert(alert);
            bsAlert.close();
        }, timeout);
    });
}

/**
 * Initialize form enhancements
 */
function initializeForms() {
    // Add loading state to submit buttons
    const forms = document.querySelectorAll('form[data-loading]');
    forms.forEach(form => {
        form.addEventListener('submit', function(e) {
            const submitButton = form.querySelector('button[type="submit"]');
            if (submitButton) {
                submitButton.disabled = true;
                submitButton.innerHTML = '<span class="spinner-border spinner-border-sm me-2"></span>Loading...';
            }
        });
    });

    // Auto-resize textareas
    const textareas = document.querySelectorAll('textarea[data-auto-resize]');
    textareas.forEach(textarea => {
        textarea.addEventListener('input', function() {
            this.style.height = 'auto';
            this.style.height = this.scrollHeight + 'px';
        });
    });
}

/**
 * Initialize DataTables if jQuery is available
 */
function initializeDataTables() {
    if (typeof $ !== 'undefined' && $.fn.DataTable) {
        $('.data-table').each(function() {
            const table = $(this);
            const options = {
                responsive: true,
                pageLength: 25,
                language: {
                    search: "Search:",
                    lengthMenu: "Show _MENU_ entries",
                    info: "Showing _START_ to _END_ of _TOTAL_ entries",
                    paginate: {
                        first: "First",
                        last: "Last",
                        next: "Next",
                        previous: "Previous"
                    }
                }
            };
            
            // Merge custom options if provided
            const customOptions = table.data('options');
            if (customOptions) {
                Object.assign(options, customOptions);
            }
            
            table.DataTable(options);
        });
    }
}

/**
 * Initialize file upload handlers
 */
function initializeFileUploads() {
    const fileInputs = document.querySelectorAll('.file-upload-wrapper input[type="file"]');
    fileInputs.forEach(input => {
        input.addEventListener('change', function() {
            const wrapper = this.closest('.file-upload-wrapper');
            const label = wrapper.querySelector('.file-upload-button');
            const fileName = this.files[0] ? this.files[0].name : 'Choose file';
            if (label) {
                label.textContent = fileName;
            }
        });
    });
}

/**
 * Utility function to get cookie value
 */
function getCookie(name) {
    const value = `; ${document.cookie}`;
    const parts = value.split(`; ${name}=`);
    if (parts.length === 2) return parts.pop().split(';').shift();
}

/**
 * API helper functions
 */
const API = {
    /**
     * Make authenticated API request
     */
    request: async function(url, options = {}) {
        const defaultOptions = {
            headers: {
                'Content-Type': 'application/json'
            },
            credentials: 'include' // This ensures cookies are sent with requests
        };
        
        const finalOptions = {
            ...defaultOptions,
            ...options,
            headers: {
                ...defaultOptions.headers,
                ...options.headers
            }
        };
        
        try {
            const response = await fetch(url, finalOptions);
            
            if (response.status === 401) {
                // Unauthorized - redirect to login
                window.location.href = '/login';
                return;
            }
            
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            
            return await response.json();
        } catch (error) {
            console.error('API request failed:', error);
            throw error;
        }
    },

    /**
     * GET request
     */
    get: function(url) {
        return this.request(url, { method: 'GET' });
    },

    /**
     * POST request
     */
    post: function(url, data) {
        return this.request(url, {
            method: 'POST',
            body: JSON.stringify(data)
        });
    },

    /**
     * PUT request
     */
    put: function(url, data) {
        return this.request(url, {
            method: 'PUT',
            body: JSON.stringify(data)
        });
    },

    /**
     * DELETE request
     */
    delete: function(url) {
        return this.request(url, { method: 'DELETE' });
    }
};

/**
 * Notification helper
 */
const Notification = {
    show: function(message, type = 'info') {
        const alertDiv = document.createElement('div');
        alertDiv.className = `alert alert-${type} alert-dismissible fade show`;
        alertDiv.innerHTML = `
            ${message}
            <button type="button" class="btn-close" data-bs-dismiss="alert"></button>
        `;
        
        const container = document.querySelector('.notification-container') || document.body;
        container.appendChild(alertDiv);
        
        // Auto-dismiss after 5 seconds
        setTimeout(() => {
            if (alertDiv.parentNode) {
                const bsAlert = new bootstrap.Alert(alertDiv);
                bsAlert.close();
            }
        }, 5000);
    },

    success: function(message) {
        this.show(message, 'success');
    },

    error: function(message) {
        this.show(message, 'danger');
    },

    warning: function(message) {
        this.show(message, 'warning');
    },

    info: function(message) {
        this.show(message, 'info');
    }
};

/**
 * Modal helper
 */
const Modal = {
    show: function(modalId) {
        const modal = document.getElementById(modalId);
        if (modal) {
            const bsModal = new bootstrap.Modal(modal);
            bsModal.show();
        }
    },

    hide: function(modalId) {
        const modal = document.getElementById(modalId);
        if (modal) {
            const bsModal = bootstrap.Modal.getInstance(modal);
            if (bsModal) {
                bsModal.hide();
            }
        }
    },

    confirm: function(message, callback) {
        const result = confirm(message);
        if (result && callback) {
            callback();
        }
        return result;
    }
};

/**
 * Form helper
 */
const Form = {
    serialize: function(form) {
        const formData = new FormData(form);
        const data = {};
        for (let [key, value] of formData.entries()) {
            data[key] = value;
        }
        return data;
    },

    reset: function(formId) {
        const form = document.getElementById(formId);
        if (form) {
            form.reset();
        }
    },

    validate: function(form) {
        const inputs = form.querySelectorAll('input[required], select[required], textarea[required]');
        let isValid = true;
        
        inputs.forEach(input => {
            if (!input.value.trim()) {
                input.classList.add('is-invalid');
                isValid = false;
            } else {
                input.classList.remove('is-invalid');
            }
        });
        
        return isValid;
    }
};

/**
 * Loading helper
 */
const Loading = {
    show: function(element) {
        if (typeof element === 'string') {
            element = document.querySelector(element);
        }
        if (element) {
            element.classList.add('loading');
            element.style.position = 'relative';
            element.style.pointerEvents = 'none';
            
            const spinner = document.createElement('div');
            spinner.className = 'loading-spinner';
            spinner.innerHTML = '<div class="spinner-border" role="status"><span class="visually-hidden">Loading...</span></div>';
            spinner.style.cssText = `
                position: absolute;
                top: 50%;
                left: 50%;
                transform: translate(-50%, -50%);
                z-index: 1000;
            `;
            element.appendChild(spinner);
        }
    },

    hide: function(element) {
        if (typeof element === 'string') {
            element = document.querySelector(element);
        }
        if (element) {
            element.classList.remove('loading');
            element.style.pointerEvents = '';
            
            const spinner = element.querySelector('.loading-spinner');
            if (spinner) {
                spinner.remove();
            }
        }
    }
};

/**
 * Utility functions
 */
const Utils = {
    formatDate: function(dateString) {
        const date = new Date(dateString);
        return date.toLocaleDateString('en-US', {
            year: 'numeric',
            month: 'short',
            day: 'numeric'
        });
    },

    formatDateTime: function(dateString) {
        const date = new Date(dateString);
        return date.toLocaleString('en-US', {
            year: 'numeric',
            month: 'short',
            day: 'numeric',
            hour: '2-digit',
            minute: '2-digit'
        });
    },

    debounce: function(func, wait) {
        let timeout;
        return function executedFunction(...args) {
            const later = () => {
                clearTimeout(timeout);
                func(...args);
            };
            clearTimeout(timeout);
            timeout = setTimeout(later, wait);
        };
    },

    throttle: function(func, limit) {
        let inThrottle;
        return function() {
            const args = arguments;
            const context = this;
            if (!inThrottle) {
                func.apply(context, args);
                inThrottle = true;
                setTimeout(() => inThrottle = false, limit);
            }
        };
    },

    copyToClipboard: function(text) {
        navigator.clipboard.writeText(text).then(function() {
            Notification.success('Copied to clipboard');
        }, function() {
            Notification.error('Failed to copy to clipboard');
        });
    }
};

// Export to global scope
window.API = API;
window.Notification = Notification;
window.Modal = Modal;
window.Form = Form;
window.Loading = Loading;
window.Utils = Utils;