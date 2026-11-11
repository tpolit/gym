use domain::{Category, Equipment, ExerciseId, MeasureKind, MuscleGroup};
use features::exercise_catalog::{
    ExerciseForm, catalog_error_message, create_custom_exercise, exercise_detail,
    update_custom_exercise,
};
use features::shared::labels::{
    category_label, equipment_label, measure_label, muscle_group_label,
};
use leptos::prelude::*;

use crate::navigation::{Navigation, Screen};
use crate::store::Store;
use crate::ui::{TitleVisibility, TopBar};

#[component]
pub fn CustomExerciseFormPage(
    store: Store,
    navigation: Navigation,
    custom_id: Option<u32>,
) -> impl IntoView {
    let initial = custom_id
        .and_then(|id| store.with_untracked(|db| ExerciseForm::of_custom_exercise(db, id)))
        .unwrap_or_else(ExerciseForm::blank);
    let measure_locked = custom_id.is_some_and(|id| {
        store.with_untracked(|db| {
            exercise_detail(db, &ExerciseId::Custom(id))
                .and_then(|detail| detail.custom)
                .is_some_and(|custom| custom.is_used)
        })
    });
    let form = RwSignal::new(initial);
    let error = RwSignal::new(None::<&'static str>);

    let save = move || {
        let current = form.get_untracked();
        let result = match custom_id {
            Some(id) => store
                .execute(|db| update_custom_exercise(db, id, &current))
                .map(|()| None),
            None => store
                .execute(|db| create_custom_exercise(db, &current))
                .map(Some),
        };
        match result {
            Ok(Some(created)) => navigation.replace(Screen::ExerciseDetail(created)),
            Ok(None) => navigation.back(),
            Err(failure) => error.set(Some(catalog_error_message(&failure))),
        }
    };

    let page_title = if custom_id.is_some() {
        "Modifier l'exercice"
    } else {
        "Nouvel exercice"
    };
    view! {
        <main class="screen">
            <TopBar on_back=Callback::new(move |()| navigation.back()) title=page_title title_visibility=TitleVisibility::OnceScrolled />
            <h1 class="title">{page_title}</h1>
            <form
                class="form"
                on:submit=move |event| {
                    event.prevent_default();
                    save();
                }
            >
                <label class="field">
                    <span class="eyebrow">"Nom"</span>
                    <input
                        class="input"
                        type="text"
                        placeholder="Rowing machine Technogym"
                        prop:value=move || form.with(|form| form.name.clone())
                        on:input=move |event| form.update(|form| form.name = event_target_value(&event))
                    />
                </label>
                <Choice
                    label="Catégorie"
                    options=Category::ALL.into_iter().map(|category| (category, category_label(category))).collect()
                    selected=Signal::derive(move || form.with(|form| form.category))
                    on_select=Callback::new(move |category| form.update(|form| form.set_category(category)))
                    disabled=Signal::derive(|| false)
                />
                <Choice
                    label="Type de mesure"
                    options=MeasureKind::ALL.into_iter().map(|measure| (measure, measure_label(measure))).collect()
                    selected=Signal::derive(move || form.with(|form| form.measure))
                    on_select=Callback::new(move |measure| {
                        form.update(|form| {
                            if form.allowed_measures().contains(&measure) {
                                form.measure = measure;
                            }
                        })
                    })
                    disabled=Signal::derive(move || measure_locked)
                />
                {measure_locked.then(|| view! {
                    <p class="muted">"Exercice déjà utilisé : le type de mesure ne peut plus changer."</p>
                })}
                <Choice
                    label="Groupe musculaire"
                    options=MuscleGroup::ALL.into_iter().map(|group| (group, muscle_group_label(group))).collect()
                    selected=Signal::derive(move || form.with(|form| form.muscle_group))
                    on_select=Callback::new(move |group| form.update(|form| form.muscle_group = group))
                    disabled=Signal::derive(|| false)
                />
                <Choice
                    label="Matériel"
                    options=Equipment::ALL.into_iter().map(|equipment| (equipment, equipment_label(equipment))).collect()
                    selected=Signal::derive(move || form.with(|form| form.equipment))
                    on_select=Callback::new(move |equipment| form.update(|form| form.equipment = equipment))
                    disabled=Signal::derive(|| false)
                />
                {move || error.get().map(|message| view! { <p class="error" role="alert">{message}</p> })}
                <button type="submit" class="btn btn--primary">"Enregistrer"</button>
            </form>
        </main>
    }
}

#[component]
fn Choice<T>(
    label: &'static str,
    options: Vec<(T, &'static str)>,
    selected: Signal<T>,
    on_select: Callback<T>,
    disabled: Signal<bool>,
) -> impl IntoView
where
    T: Copy + PartialEq + Send + Sync + 'static,
{
    let values: Vec<T> = options.iter().map(|(value, _)| *value).collect();
    view! {
        <label class="field">
            <span class="eyebrow">{label}</span>
            <select
                class="input"
                disabled=move || disabled.get()
                on:change=move |event| {
                    if let Some(value) = event_target_value(&event).parse::<usize>().ok().and_then(|index| values.get(index)) {
                        on_select.run(*value);
                    }
                }
            >
                {options
                    .into_iter()
                    .enumerate()
                    .map(|(index, (value, text))| {
                        view! {
                            <option value=index.to_string() selected=move || selected.get() == value>
                                {text}
                            </option>
                        }
                    })
                    .collect_view()}
            </select>
        </label>
    }
}
