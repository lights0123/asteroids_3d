use std::collections::HashMap;

use bevy_rapier3d::rapier::math::Vector;
use bevy_rapier3d::rapier::pipeline::{
    ContactModificationContext, PhysicsHooks, PhysicsHooksFlags,
};

pub struct OneWayPlatformHook {
    pub platforms: HashMap<u64, Vector<f32>>,
}

impl PhysicsHooks for OneWayPlatformHook {
    fn active_hooks(&self) -> PhysicsHooksFlags {
        PhysicsHooksFlags::MODIFY_SOLVER_CONTACTS
    }

    fn modify_solver_contacts(&self, context: &mut ContactModificationContext) {
        // The allowed normal for the first platform is its local +y axis, and the
        // allowed normal for the second platform is its local -y axis.
        //
        // Now we have to be careful because the `manifold.local_n1` normal points
        // toward the outside of the shape of `context.co1`. So we need to flip the
        // allowed normal direction if the platform is in `context.collider_handle2`.
        //
        // Therefore:
        // - If context.collider_handle1 == self.platform1 then the allowed normal is +y.
        // - If context.collider_handle2 == self.platform1 then the allowed normal is -y.
        // - If context.collider_handle1 == self.platform2 then its allowed normal +y needs to be flipped to -y.
        // - If context.collider_handle2 == self.platform2 then the allowed normal -y needs to be flipped to +y.
        let mut allowed_local_n1 = Vector::zeros();
        for (id, vec) in self.platforms.iter() {
            if context.collider1.user_data as u64 == *id {
                allowed_local_n1 = vec.clone_owned();
                break;
            }
            if context.collider2.user_data as u64 == *id {
                allowed_local_n1 = -vec.clone_owned();
                break;
            }
        }

        // Call the helper function that simulates one-way platforms.
        context.update_as_oneway_platform(&allowed_local_n1, 0.1);
    }
}
