STOW ?= stow
HOME_DIR ?= $(HOME)

.PHONY: stow unstow stow-bin stow-ghostty stow-kak stow-yazi unstow-bin unstow-ghostty unstow-kak unstow-yazi

stow:
	$(MAKE) stow-bin
	$(MAKE) stow-ghostty
	$(MAKE) stow-kak
	$(STOW) tmux
	$(MAKE) stow-yazi

unstow:
	$(MAKE) unstow-bin
	$(MAKE) unstow-ghostty
	$(MAKE) unstow-kak
	$(STOW) -D tmux
	$(MAKE) unstow-yazi

stow-bin:
	install -d $(HOME_DIR)/.local/bin
	$(STOW) -t $(HOME_DIR)/.local/bin bin

unstow-bin:
	$(STOW) -D -t $(HOME_DIR)/.local/bin bin

stow-ghostty:
	install -d $(HOME_DIR)/.config/ghostty
	$(STOW) -t $(HOME_DIR)/.config/ghostty ghostty

unstow-ghostty:
	$(STOW) -D -t $(HOME_DIR)/.config/ghostty ghostty

stow-kak:
	install -d $(HOME_DIR)/.config/kak
	$(STOW) -t $(HOME_DIR)/.config/kak kak

unstow-kak:
	$(STOW) -D -t $(HOME_DIR)/.config/kak kak

stow-yazi:
	install -d $(HOME_DIR)/.config/yazi
	$(STOW) -t $(HOME_DIR)/.config/yazi yazi

unstow-yazi:
	$(STOW) -D -t $(HOME_DIR)/.config/yazi yazi
