

mod game;

use eframe::egui;
use game::{Deck, Hand};

#[cfg(target_os = "android")]
use android_activity::AndroidApp;

pub fn run_app() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Blackjack",
        options,
        Box::new(|_cc| Ok(Box::new(BlackjackApp::new()))),
    )
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "full");
    }
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );

    let options = eframe::NativeOptions {
        android_app: Some(app),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Blackjack",
        options,
        Box::new(|_cc| Ok(Box::new(BlackjackApp::new()))),
    );
}

enum GameState {
    PlayerTurn,
    DealerTurn,
    RoundEnd,
}

struct BlackjackApp {
    state: GameState,
    deck: Deck,
    player_hand: Hand,
    dealer_hand: Hand,
    round_result: String,
}

impl BlackjackApp {
    fn new() -> Self {
        let mut deck = Deck::new();
        deck.shuffle();

        let mut player_hand = Hand::new();
        let mut dealer_hand = Hand::new();

        for _ in 0..2 {
            player_hand.add_card(deck.deal().unwrap());
            dealer_hand.add_card(deck.deal().unwrap());
        }

        let mut app = Self {
            state: GameState::PlayerTurn,
            deck,
            player_hand,
            dealer_hand,
            round_result: String::new(),
        };

        if app.player_hand.value() == 21 {
            app.resolve_dealer_turn();
        }

        app
    }

    fn reset_round(&mut self) {
        self.deck = Deck::new();
        self.deck.shuffle();
        self.player_hand = Hand::new();
        self.dealer_hand = Hand::new();

        for _ in 0..2 {
            self.player_hand.add_card(self.deck.deal().unwrap());
            self.dealer_hand.add_card(self.deck.deal().unwrap());
        }

        self.state = GameState::PlayerTurn;
        self.round_result = String::new();

        if self.player_hand.value() == 21 {
            self.resolve_dealer_turn();
        }
    }

    fn handle_hit(&mut self) {
        let new_card = self.deck.deal().unwrap();
        self.player_hand.add_card(new_card);

        if self.player_hand.value() > 21 {
            self.round_result = String::from("BUST! You lose this round.");
            self.state = GameState::RoundEnd;
        } else if self.player_hand.value() == 21 {
            self.resolve_dealer_turn();
        }
    }

    fn handle_stand(&mut self) {
        self.resolve_dealer_turn();
    }

    fn resolve_dealer_turn(&mut self) {
        self.state = GameState::DealerTurn;

        while self.dealer_hand.value() < 17 {
            let new_card = self.deck.deal().unwrap();
            self.dealer_hand.add_card(new_card);
        }

        let player_score = self.player_hand.value();
        let dealer_score = self.dealer_hand.value();

        if dealer_score > 21 {
            self.round_result = String::from("Dealer busts! You win!");
        } else if player_score > dealer_score {
            self.round_result = format!("You win! ({} vs {})", player_score, dealer_score);
        } else if player_score < dealer_score {
            self.round_result = format!("You lose. ({} vs {})", player_score, dealer_score);
        } else {
            self.round_result = format!("Push! It's a tie at {}", player_score);
        }

        self.state = GameState::RoundEnd;
    }
}

impl eframe::App for BlackjackApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);

                ui.heading(egui::RichText::new("♠ BLACKJACK ♥").size(24.0));

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);

                ui.heading("DEALER");
                ui.add_space(5.0);

                match self.state {
                    GameState::PlayerTurn => {
                        let cards = self.dealer_hand.display_str();
                        let visible = cards.split_once(' ').map(|(_, rest)| rest).unwrap_or("");
                        ui.label(format!("Cards: [??] {}", visible));
                        ui.label("Value: ???");
                    }
                    _ => {
                        ui.label(format!("Cards: {}", self.dealer_hand.display_str()));
                        ui.label(format!("Value: {}", self.dealer_hand.value()));
                    }
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                ui.heading("PLAYER");
                ui.add_space(5.0);
                ui.label(format!("Cards: {}", self.player_hand.display_str()));
                ui.label(format!("Value: {}", self.player_hand.value()));

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(20.0);

                match self.state {
                    GameState::PlayerTurn => {
                        ui.horizontal(|ui| {
                            let button_size = egui::vec2(120.0, 40.0);
                            let spacing = ui.style().spacing.item_spacing.x;
                            let total_buttons_width = button_size.x * 2.0 + spacing;

                            let available_width = ui.available_rect_before_wrap().width();
                            let spacer_width = (available_width - total_buttons_width) / 2.0;
                            if spacer_width > 0.0 {
                                ui.add_space(spacer_width);
                            }

                            if ui
                                .add(egui::Button::new("Hit").min_size(button_size))
                                .clicked()
                            {
                                self.handle_hit();
                            }
                            if ui
                                .add(egui::Button::new("Stand").min_size(button_size))
                                .clicked()
                            {
                                self.handle_stand();
                            }
                        });
                    }
                    GameState::DealerTurn => {
                        ui.label("Dealer is playing...");
                    }
                    GameState::RoundEnd => {
                        ui.label(
                            egui::RichText::new(&self.round_result)
                                .size(18.0)
                                .strong(),
                        );
                        ui.add_space(10.0);

                        let button_size = egui::vec2(250.0, 40.0);
                        if ui
                            .add(egui::Button::new("New Round").min_size(button_size))
                            .clicked()
                        {
                            self.reset_round();
                        }
                    }
                }
            });
        });
    }
}
