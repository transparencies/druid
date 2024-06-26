// Copyright 2020 the Druid Authors
// SPDX-License-Identifier: Apache-2.0

//! A widget that can dynamically switch between one of many views.

use crate::widget::prelude::*;
use crate::{Data, Point, WidgetPod};
use tracing::instrument;

type ChildPicker<T, U> = dyn Fn(&T, &Env) -> U;
type ChildBuilder<T, U> = dyn Fn(&U, &T, &Env) -> Box<dyn Widget<T>>;

/// A widget that switches dynamically between multiple children.
pub struct ViewSwitcher<T, U> {
    child_picker: Box<ChildPicker<T, U>>,
    child_builder: Box<ChildBuilder<T, U>>,
    active_child: Option<WidgetPod<T, Box<dyn Widget<T>>>>,
    active_child_id: Option<U>,
}

impl<T: Data, U: Data> ViewSwitcher<T, U> {
    /// Create a new view switcher.
    ///
    /// The `child_picker` closure is called every time the application data changes.
    /// If the value it returns is the same as the one it returned during the previous
    /// data change, nothing happens. If it returns a different value, then the
    /// `child_builder` closure is called with the new value.
    ///
    /// The `child_builder` closure creates a new child widget based on
    /// the value passed to it.
    ///
    /// # Examples
    /// ```
    /// use druid::{
    ///     widget::{Label, ViewSwitcher},
    ///     Data, Widget,
    /// };
    ///
    /// #[derive(Clone, PartialEq, Data)]
    /// enum Foo {
    ///     A,
    ///     B,
    ///     C,
    /// }
    ///
    /// fn ui() -> impl Widget<Foo> {
    ///     ViewSwitcher::new(
    ///         |data: &Foo, _env| data.clone(),
    ///         |selector, _data, _env| match selector {
    ///             Foo::A => Box::new(Label::new("A")),
    ///             _ => Box::new(Label::new("Not A")),
    ///         },
    ///     )
    /// }
    /// ```
    pub fn new(
        child_picker: impl Fn(&T, &Env) -> U + 'static,
        child_builder: impl Fn(&U, &T, &Env) -> Box<dyn Widget<T>> + 'static,
    ) -> Self {
        Self {
            child_picker: Box::new(child_picker),
            child_builder: Box::new(child_builder),
            active_child: None,
            active_child_id: None,
        }
    }
}

impl<T: Data, U: Data> Widget<T> for ViewSwitcher<T, U> {
    #[instrument(
        name = "ViewSwitcher",
        level = "trace",
        skip(self, ctx, event, data, env)
    )]
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        if let Some(child) = self.active_child.as_mut() {
            child.event(ctx, event, data, env);
        }
    }

    #[instrument(
        name = "ViewSwitcher",
        level = "trace",
        skip(self, ctx, event, data, env)
    )]
    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            let child_id = (self.child_picker)(data, env);
            self.active_child = Some(WidgetPod::new((self.child_builder)(&child_id, data, env)));
            self.active_child_id = Some(child_id);
        }
        if let Some(child) = self.active_child.as_mut() {
            child.lifecycle(ctx, event, data, env);
        }
    }

    #[instrument(
        name = "ViewSwitcher",
        level = "trace",
        skip(self, ctx, _old_data, data, env)
    )]
    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        let child_id = (self.child_picker)(data, env);
        // Safe to unwrap because self.active_child_id should not be empty
        if !child_id.same(self.active_child_id.as_ref().unwrap()) {
            self.active_child = Some(WidgetPod::new((self.child_builder)(&child_id, data, env)));
            self.active_child_id = Some(child_id);
            ctx.children_changed();
        // Because the new child has not yet been initialized, we have to skip the update after switching.
        } else if let Some(child) = self.active_child.as_mut() {
            child.update(ctx, data, env);
        }
    }

    #[instrument(name = "ViewSwitcher", level = "trace", skip(self, ctx, bc, data, env))]
    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        match self.active_child {
            Some(ref mut child) => {
                let size = child.layout(ctx, bc, data, env);
                child.set_origin(ctx, Point::ORIGIN);
                size
            }
            None => bc.max(),
        }
    }

    #[instrument(name = "ViewSwitcher", level = "trace", skip(self, ctx, data, env))]
    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        if let Some(ref mut child) = self.active_child {
            child.paint_raw(ctx, data, env);
        }
    }
}
